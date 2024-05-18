/// Design for a table with 5 fields in two indices
///
/// ```
/// table! {
///     primary {
///          mut a: i32,
///              b: i32,
///          mut c: i32,
///     },
///     assoc {
///             d: i32,
///         mut e: i32,
///     }
/// }
/// ```
/// 
/// 
/// 