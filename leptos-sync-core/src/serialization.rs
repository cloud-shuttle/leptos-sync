//! Serialization utilities for efficient data transmission and storage

use serde::{Deserialize, Serialize};
use std::io;

/// Serialization format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    /// Binary format - fast and compact
    Bincode,
    /// JSON format - human readable and widely supported
    Json,
}

impl Default for SerializationFormat {
    fn default() -> Self {
        Self::Bincode
    }
}

/// Serialization error types
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("Bincode serialization error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

/// High-performance serialization utilities
pub struct Serializer {
    format: SerializationFormat,
}

impl Serializer {
    /// Create a new serializer with the specified format
    pub fn new(format: SerializationFormat) -> Self {
        Self { format }
    }

    /// Create a serializer with the default format (bincode)
    pub fn default() -> Self {
        Self::new(SerializationFormat::default())
    }

    /// Serialize data to bytes
    pub fn serialize<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, SerializationError> {
        match self.format {
            SerializationFormat::Bincode => {
                bincode::serialize(value).map_err(SerializationError::Bincode)
            }
            SerializationFormat::Json => {
                serde_json::to_vec(value).map_err(SerializationError::Json)
            }
        }
    }

    /// Deserialize data from bytes
    pub fn deserialize<T: for<'de> Deserialize<'de>>(
        &self,
        bytes: &[u8],
    ) -> Result<T, SerializationError> {
        match self.format {
            SerializationFormat::Bincode => {
                bincode::deserialize(bytes).map_err(SerializationError::Bincode)
            }
            SerializationFormat::Json => {
                serde_json::from_slice(bytes).map_err(SerializationError::Json)
            }
        }
    }

    /// Get the current serialization format
    pub fn format(&self) -> SerializationFormat {
        self.format
    }

    /// Change the serialization format
    pub fn set_format(&mut self, format: SerializationFormat) {
        self.format = format;
    }
}

/// Optimized serialization for CRDTs
pub struct CRDTSerializer {
    serializer: Serializer,
    compression_enabled: bool,
}

impl CRDTSerializer {
    /// Create a new CRDT serializer with bincode and compression
    pub fn new() -> Self {
        Self {
            serializer: Serializer::new(SerializationFormat::Bincode),
            compression_enabled: true,
        }
    }

    /// Create a CRDT serializer with custom settings
    pub fn with_settings(format: SerializationFormat, compression: bool) -> Self {
        Self {
            serializer: Serializer::new(format),
            compression_enabled: compression,
        }
    }

    /// Serialize a CRDT with optional compression
    pub fn serialize_crdt<T: Serialize>(&self, value: &T) -> Result<Vec<u8>, SerializationError> {
        let serialized = self.serializer.serialize(value)?;

        if self.compression_enabled && serialized.len() > 1024 {
            // Only compress if data is larger than 1KB
            self.compress(&serialized)
        } else {
            Ok(serialized)
        }
    }

    /// Deserialize a CRDT with automatic decompression detection
    pub fn deserialize_crdt<T: for<'de> Deserialize<'de>>(
        &self,
        bytes: &[u8],
    ) -> Result<T, SerializationError> {
        if self.is_compressed(bytes) {
            let decompressed = self.decompress(bytes)?;
            self.serializer.deserialize(&decompressed)
        } else {
            self.serializer.deserialize(bytes)
        }
    }

    /// Check if data is compressed
    fn is_compressed(&self, bytes: &[u8]) -> bool {
        // Simple heuristic: check if data looks like compressed data
        bytes.len() > 0 && bytes[0] == 0x78 && bytes[1] == 0x9C
    }

    /// Compress data using flate2
    #[cfg(not(target_arch = "wasm32"))]
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SerializationError> {
        #[cfg(feature = "compression")]
        {
            use flate2::write::DeflateEncoder;
            use flate2::Compression;
            use std::io::Write;

            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(data)?;
            Ok(encoder.finish()?)
        }
        #[cfg(not(feature = "compression"))]
        {
            // Return uncompressed data when compression is not available
            Ok(data.to_vec())
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, SerializationError> {
        // For WASM, return uncompressed data for now
        // TODO: Implement WASM-compatible compression
        Ok(data.to_vec())
    }

    /// Decompress data using flate2
    #[cfg(not(target_arch = "wasm32"))]
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SerializationError> {
        #[cfg(feature = "compression")]
        {
            use flate2::read::DeflateDecoder;
            use std::io::Read;

            let mut decoder = DeflateDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;
            Ok(decompressed)
        }
        #[cfg(not(feature = "compression"))]
        {
            // Return data as-is when compression is not available
            Ok(data.to_vec())
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, SerializationError> {
        // For WASM, return data as-is for now
        // TODO: Implement WASM-compatible decompression
        Ok(data.to_vec())
    }
}

impl Default for CRDTSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance comparison utilities
pub mod benchmark {
    use super::*;
    use std::time::Instant;

    /// Benchmark serialization performance
    pub fn benchmark_serialization<T: Serialize + for<'de> Deserialize<'de>>(
        data: &T,
        iterations: usize,
    ) -> SerializationBenchmark {
        let mut bincode_serializer = Serializer::new(SerializationFormat::Bincode);
        let mut json_serializer = Serializer::new(SerializationFormat::Json);

        // Warm up
        let _ = bincode_serializer.serialize(data);
        let _ = json_serializer.serialize(data);

        // Benchmark bincode
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = bincode_serializer.serialize(data);
        }
        let bincode_serialize_time = start.elapsed();

        let serialized_bincode = bincode_serializer.serialize(data).unwrap();
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = bincode_serializer.deserialize::<T>(&serialized_bincode);
        }
        let bincode_deserialize_time = start.elapsed();

        // Benchmark JSON
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = json_serializer.serialize(data);
        }
        let json_serialize_time = start.elapsed();

        let serialized_json = json_serializer.serialize(data).unwrap();
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = json_serializer.deserialize::<T>(&serialized_json);
        }
        let json_deserialize_time = start.elapsed();

        SerializationBenchmark {
            bincode_serialize_time,
            bincode_deserialize_time,
            json_serialize_time,
            json_deserialize_time,
            bincode_size: serialized_bincode.len(),
            json_size: serialized_json.len(),
            iterations,
        }
    }

    /// Serialization benchmark results
    #[derive(Debug)]
    pub struct SerializationBenchmark {
        pub bincode_serialize_time: std::time::Duration,
        pub bincode_deserialize_time: std::time::Duration,
        pub json_serialize_time: std::time::Duration,
        pub json_deserialize_time: std::time::Duration,
        pub bincode_size: usize,
        pub json_size: usize,
        pub iterations: usize,
    }

    impl SerializationBenchmark {
        /// Get the performance improvement ratio
        pub fn serialize_improvement_ratio(&self) -> f64 {
            self.json_serialize_time.as_nanos() as f64
                / self.bincode_serialize_time.as_nanos() as f64
        }

        /// Get the size reduction ratio
        pub fn size_reduction_ratio(&self) -> f64 {
            self.json_size as f64 / self.bincode_size as f64
        }

        /// Print a formatted summary
        pub fn print_summary(&self) {
            println!("=== Serialization Benchmark Results ===");
            println!("Iterations: {}", self.iterations);
            println!();
            println!("Bincode:");
            println!("  Serialize:   {:?}", self.bincode_serialize_time);
            println!("  Deserialize: {:?}", self.bincode_deserialize_time);
            println!("  Size:        {} bytes", self.bincode_size);
            println!();
            println!("JSON:");
            println!("  Serialize:   {:?}", self.json_serialize_time);
            println!("  Deserialize: {:?}", self.json_deserialize_time);
            println!("  Size:        {} bytes", self.json_size);
            println!();
            println!("Improvements:");
            println!(
                "  Serialize:   {:.2}x faster",
                self.serialize_improvement_ratio()
            );
            println!("  Size:        {:.2}x smaller", self.size_reduction_ratio());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crdt::{LwwRegister, ReplicaId};

    #[test]
    fn test_serialization_formats() {
        let data = LwwRegister::new("test_value".to_string(), ReplicaId::default());

        let bincode_serializer = Serializer::new(SerializationFormat::Bincode);
        let json_serializer = Serializer::new(SerializationFormat::Json);

        let bincode_bytes = bincode_serializer.serialize(&data).unwrap();
        let json_bytes = json_serializer.serialize(&data).unwrap();

        let bincode_deserialized: LwwRegister<String> =
            bincode_serializer.deserialize(&bincode_bytes).unwrap();
        let json_deserialized: LwwRegister<String> =
            json_serializer.deserialize(&json_bytes).unwrap();

        assert_eq!(data, bincode_deserialized);
        assert_eq!(data, json_deserialized);
        assert!(bincode_bytes.len() < json_bytes.len());
    }

    #[test]
    fn test_crdt_serializer() {
        let data = LwwRegister::new("test_value".to_string(), ReplicaId::default());
        let serializer = CRDTSerializer::new();

        let serialized = serializer.serialize_crdt(&data).unwrap();
        let deserialized: LwwRegister<String> = serializer.deserialize_crdt(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_serialization_benchmark() {
        let data = LwwRegister::new("test_value".to_string(), ReplicaId::default());
        let benchmark = benchmark::benchmark_serialization(&data, 1000);

        // Bincode should be faster and smaller
        assert!(benchmark.serialize_improvement_ratio() > 1.0);
        assert!(benchmark.size_reduction_ratio() > 1.0);
    }
}
