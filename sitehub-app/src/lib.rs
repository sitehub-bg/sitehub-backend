//! Domain, ports, and use cases for sitehub.
//!
//! This package is the inside of the hexagon: it must not depend on any I/O,
//! framework, or specific database. Adapters depend on this package; this
//! package depends on no other internal package.

#[cfg(test)]
mod tests {
    #[test]
    fn app_crate_loads() {}
}
