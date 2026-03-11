// ish-runtime: Minimal value type for compiled ish functions.
//
// Compiled functions work with this simple enum across the FFI boundary.
// No GC dependency — values are owned.

/// The value type used by compiled ish functions across the FFI boundary.
/// Keep this simple and #[repr(C)]-compatible for dynamic loading.
#[derive(Debug, Clone)]
pub enum IshValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    Null,
}

impl IshValue {
    pub fn as_i64(&self) -> i64 {
        match self {
            IshValue::Int(n) => *n,
            IshValue::Float(f) => *f as i64,
            IshValue::Bool(b) => if *b { 1 } else { 0 },
            IshValue::Null => 0,
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            IshValue::Float(f) => *f,
            IshValue::Int(n) => *n as f64,
            IshValue::Bool(b) => if *b { 1.0 } else { 0.0 },
            IshValue::Null => 0.0,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            IshValue::Bool(b) => *b,
            IshValue::Int(n) => *n != 0,
            IshValue::Float(f) => *f != 0.0,
            IshValue::Null => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversions() {
        assert_eq!(IshValue::Int(42).as_i64(), 42);
        assert_eq!(IshValue::Float(3.14).as_i64(), 3);
        assert!(IshValue::Int(1).is_truthy());
        assert!(!IshValue::Null.is_truthy());
    }
}
