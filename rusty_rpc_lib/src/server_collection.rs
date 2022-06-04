use std::collections::{hash_map::Entry, HashMap};
use std::mem::transmute;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread::panicking;

use tokio::sync::{Mutex, MutexGuard};

use crate::{messages::ServiceId, traits::RustyRpcServiceServer};

pub struct RawBox<T>(*mut T);
impl<T> RawBox<T> {
    pub unsafe fn new(value: *mut T) -> Self {
        RawBox(value)
    }
    pub fn get(&self) -> *mut T {
        self.0
    }
}
unsafe impl<T: Sync> Sync for RawBox<T> {}
unsafe impl<T: Send> Send for RawBox<T> {}

/// This acts like Box<MutexGuard<...>>, except that other people can safely
/// have references to this parent_guard while this ServerEntry is not in the
/// process of being dropped.
pub type ServerGuard = RawBox<MutexGuard<'static, ServerEntry>>;

/// Represents a server that can live for some unknown lifetime, and might
/// reference a parent server with a longer lifetime.
pub struct ServerEntry {
    /// Not actually any lifetime, but unknown lifetime.
    server_: Box<dyn for<'a> RustyRpcServiceServer<'a>>,
    /// Not actually 'static, but unknown lifetime. This field is never read
    /// from, but it matters that it's dropped when this ServerEntry is dropped.
    #[allow(dead_code)]
    parent_guard: Option<ServerGuard>,
}
impl ServerEntry {
    pub unsafe fn server(&mut self) -> &mut dyn RustyRpcServiceServer<'_> {
        &mut *self.server_
    }
}
impl Drop for ServerEntry {
    fn drop(&mut self) {
        if !panicking() {
            if let Some(guard) = &mut self.parent_guard {
                unsafe {
                    drop(Box::from_raw(guard.0));
                }
            }
        }
    }
}

/// State for one ongoing connection with one client.
pub struct ServerCollection {
    active_services: Mutex<HashMap<ServiceId, Arc<Mutex<ServerEntry>>>>,
    next_service_id: AtomicU64,
}
impl ServerCollection {
    pub(crate) fn new() -> Self {
        ServerCollection {
            active_services: Mutex::new(HashMap::new()),
            next_service_id: AtomicU64::new(0),
        }
    }

    fn get_and_increment_next_service_id(&self) -> ServiceId {
        // This wraps around on overflow
        ServiceId(self.next_service_id.fetch_add(1, Ordering::SeqCst))
    }

    /// Add a service to the collection, and return its ID.
    #[must_use]
    pub unsafe fn register_service<'a: 'service, 'service>(
        &'a self,
        service: Box<dyn RustyRpcServiceServer<'service>>,
        parent_guard: Option<ServerGuard>,
    ) -> ServiceId {
        // Keep trying new service IDs until it's available.
        // This would go into an infinite loop if all possible ServiceIds were
        // used, but we would run out of memory before that would ever happen.
        loop {
            let mut locked = self
                .active_services
                .try_lock()
                .expect("register_service lock failed");
            let curr_service_id = self.get_and_increment_next_service_id();
            match locked.entry(curr_service_id) {
                Entry::Vacant(entry) => {
                    let server_entry: ServerEntry = ServerEntry {
                        server_: transmute::<
                            Box<dyn RustyRpcServiceServer<'service>>,
                            Box<dyn for<'b> RustyRpcServiceServer<'b>>,
                        >(service),
                        parent_guard,
                    };
                    entry.insert(Arc::new(Mutex::new(server_entry)));
                    return curr_service_id;
                }
                Entry::Occupied(_) => (),
            }
        }
    }

    /// Unregisters the service with a given ID and returns it.
    pub(crate) fn remove_service_entry_arc(
        &self,
        service_id: ServiceId,
    ) -> Option<Arc<Mutex<ServerEntry>>> {
        let mut locked = self
            .active_services
            .try_lock()
            .expect("remove_service_arc lock failed");
        locked.remove(&service_id)
    }

    pub(crate) fn get_service_entry_arc(
        &self,
        service_id: ServiceId,
    ) -> Option<Arc<Mutex<ServerEntry>>> {
        let locked = self
            .active_services
            .try_lock()
            .expect("get_service_arc lock failed");
        locked.get(&service_id).cloned()
    }
}
