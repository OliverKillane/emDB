//! # Direct Control over tabl structure
//! A raw interface to allow users to choose each data type, for the

use proc_macro2::TokenStream;




/// Provides raw access to generate table structures
/// ```
/// raw_interface!{
///     primary: <column name> {
///         mut field: type,
///         field: type,
///     }
///     associated: [
///         <column name> {
///             mut field: type,
///             field: type,
///         }
///     ],
///     updates: [
///         
///     ],
///     deletions,
///     transactions,
/// }
/// ```
pub fn raw_interface(
    tks: TokenStream,
) {



} 

