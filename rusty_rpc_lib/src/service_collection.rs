use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex},
};

use crate::{messages::ServiceId, traits::RustyRpcService};

/// State for one ongoing connection with one client.
pub struct ServiceCollection {
    active_services: HashMap<ServiceId, Arc<Mutex<Box<dyn RustyRpcService>>>>,
    next_service_id: ServiceId,
}
impl ServiceCollection {
    pub(crate) fn new() -> Self {
        ServiceCollection {
            active_services: HashMap::new(),
            next_service_id: ServiceId(0),
        }
    }

    /// Add a service to the collection, and return its ID.
    #[must_use]
    pub fn register_service(&mut self, service: Box<dyn RustyRpcService>) -> ServiceId {
        // Keep trying new service IDs until it's available.
        // This would go into an infinite loop if all possible ServiceIds were
        // used, but we would run out of memory before that would ever happen.
        loop {
            match self.active_services.entry(self.next_service_id) {
                Entry::Vacant(entry) => {
                    entry.insert(Arc::new(Mutex::new(service)));
                    let curr_service_id = self.next_service_id;
                    self.next_service_id.increment();
                    return curr_service_id;
                }
                Entry::Occupied(_) => {
                    self.next_service_id.increment();
                }
            }
        }
    }

    /// Returns the service with a given ID, or None if it doesn't exist.
    pub(crate) fn get_service_arc(
        &self,
        service_id: ServiceId,
    ) -> Option<Arc<Mutex<Box<dyn RustyRpcService>>>> {
        self.active_services.get(&service_id).map(|x| x.clone())
    }
}
