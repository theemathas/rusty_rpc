use futures::{Sink, Stream};
use serde::{de::DeserializeOwned, Serialize};
use std::io;
use tokio_serde::formats::Json;
use tokio_util::codec::LengthDelimitedCodec;

/// Turns a byte I/O into a Stream/Sink.
pub fn to_stream_sink<
    T1: DeserializeOwned,
    T2: Serialize,
    RW: tokio::io::AsyncRead + tokio::io::AsyncWrite,
>(
    read_write: RW,
) -> impl Stream<Item = Result<T1, io::Error>> + Sink<T2, Error = io::Error> {
    // This implements Stream<Item=Bytes> and Sink<Bytes> (Bytes is from the bytes crate)
    let bytes_stream_sink = tokio_util::codec::Framed::new(read_write, LengthDelimitedCodec::new());
    tokio_serde::Framed::new(bytes_stream_sink, Json::<T1, T2>::default())
}
