// cells refcells and such
pub mod cell;
pub mod refcell;
pub mod rc;

// fn escape<'a>(s: &'a str) -> Cow<'a, str> {
//     use std::borrow::Cow;
//     if already_escaped(s) {
//         Cow::Borrowed(s)
//     } else {
//         let mut string = s.to_string();
//         Cow::Owned(string)
//     }
// }
// impl String {
//     fn from_utf8_lossy(bytes: &[u8]) -> Cow<'_, str> {
//         if valid_utf8(bytes) => {Cow::Borrowed(bytes as &str)  {
//              Cow::Borrowed(bytes as &str)
//         } else {
//             let mut bts = Vec::from(bytes);
//             for bts {
//                 // replace with INVALID_CHARACTER utf-8 symbol if not valid utf-8
//             }
//             Cow::Owned(bts as String)
//         }
//     }
// }