mod other;

pub fn strlen(s: impl AsRef<str>) -> usize {
    s.as_ref().len()
}

pub fn strlenb<S: AsRef<str>>(s: S) -> usize {
    s.as_ref().len()
}

pub fn strlen2<S>(s: S) -> usize
where
    S: AsRef<str>,
{
    s.as_ref().len()
}

// compiler generates these via monomorphization
// for types that used it
pub fn strlen_refstr(s: &str) -> usize {
    s.len()
}

pub fn strlen_string(s: String) -> usize {
    s.len()
}

pub trait Hei {
    fn hei(&self);

    fn weird() where Self: Sized {} // opt out of use in trait objects
}

impl Hei for &str {
    fn hei(&self) {
        println!("hei {}", self);
    }
}

impl Hei for String {
    fn hei(&self) {
        println!("hei {}", self);
    }
}

pub fn strlen_dyn(s: Box<dyn AsRef<str>>) -> usize {
    s.as_ref().as_ref().len()
}

pub fn strlen_dyn2(s: &dyn AsRef<str>) -> usize {
    s.as_ref().len()
}

// type erasure argument
// with an associated type, it must be specified for a trait object
pub fn bar(s: &[&dyn Hei]) {
    for h in s {
        h.hei();
    }
}

pub fn say_hei(s: &dyn Hei) {
    s.hei();
}

pub fn foo() {
    strlen("hello world"); // &'static str
    strlen(String::from("hei verden")); // String

    for h in vec!["J", "GG"] {
        h.hei();
    }
    bar(&[&"J", &"GG"]);
    bar(&[&String::from("J"), &String::from("GG")]);
    bar(&[&"J", &String::from("GG")]);

    let x = Box::new(String::from("hello"));
    let y: Box<dyn AsRef<str>> = x;
    strlen_dyn(y);
    
    let z: &dyn AsRef<str> = &"world";
    strlen_dyn2(z);

}



use std::borrow::Cow;
use pdf::content::Operation;
use pdf::primitive::Primitive;
use pdf::object::Page;

fn text_objects(operations: &[Operation]) -> impl Iterator<Item = TextObject<'_>> + '_ {
    TextObjectParser {
        ops: operations.iter(),
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TextObject<'src> {
    pub x: f32,
    pub y: f32,
    pub text: Cow<'src, str>,
}

#[derive(Debug, Clone)]
struct TextObjectParser<'src> {
    ops: std::slice::Iter<'src, Operation>,
}

impl<'src> Iterator for TextObjectParser<'src> {
    type Item = TextObject<'src>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut last_coords = None;
        let mut last_text = None;

        while let Some(Operation { operator, operands }) = self.ops.next() {
            match (operator.as_str(), operands.as_slice()) {
                ("BT", _) => {
                    // Clear all prior state because we've just seen a
                    // "begin text" op
                    last_coords = None;
                    last_text = None;
                }
                ("Td", [Primitive::Number(x), Primitive::Number(y)]) => {
                    // "Text Location" contains the location of the text on the
                    // current page.
                    last_coords = Some((*x, *y));
                }
                ("Tj", [Primitive::String(text)]) => {
                    // "Show text" - the operation that actually contains the
                    // text to be displayed.
                    last_text = text.as_str().ok();
                }
                ("ET", _) => {
                    // "end of text" - we should have finished this text object,
                    // if we got all the right information then we can yield it
                    // to the caller. Otherwise, use take() to clear anything
                    // we've seen so far and continue.
                    if let (Some((x, y)), Some(text)) = (last_coords.take(), last_text.take()) {
                        return Some(TextObject { x, y, text });
                    }
                }
                _ => continue,
            }
        }

        None
    }
}   

use std::{iter::Peekable, marker::PhantomData};

pub fn group_by<I, F, K>(iterator: I, grouper: F) -> impl Iterator<Item = Vec<I::Item>>
where
    I: IntoIterator,
    F: FnMut(&I::Item) -> K,
    K: PartialEq,
{
    GroupBy {
        iter: iterator.into_iter().peekable(),
        grouper,
        _key: PhantomData,
    }
}

struct GroupBy<I: Iterator, F, K> {
    iter: Peekable<I>,
    grouper: F,
    _key: PhantomData<fn() -> K>,
}

impl<I, F, K> Iterator for GroupBy<I, F, K>
where
    I: Iterator,
    F: FnMut(&I::Item) -> K,
    K: PartialEq,
{
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        let first_item = self.iter.next()?;
        let key = (self.grouper)(&first_item);

        let mut items = vec![first_item];

        while let Some(peek) = self.iter.peek() {
            if (self.grouper)(peek) != key {
                break;
            }

            items.push(
                self.iter
                    .next()
                    .expect("Peek guarantees there is another item"),
            );
        }

        Some(items)
    }
}

pub struct ContactList {
    pub members: Vec<MemberInfo>,
}

pub struct MemberInfo {
    pub first_name: String,
    pub surname: String,
    pub email: String,
    pub mobile: String,
}

fn parse_members_on_page(page: &Page) -> Result<Vec<MemberInfo>, Error> {
    let content = match &page.contents {
        Some(c) => c,
        None => return Ok(Vec::new()),
    };

    let text_objects = text_objects(&content.operations);

    let rows = group_by(text_objects, |t| t.y)
        // ignore everything up to the table header
        .skip_while(|row| row[0].text != "Surname")
        // then skip the header
        .skip(1)
        // every row in the contact table is guaranteed to have 6 cells
        .take_while(|row| row.len() == 6);

    let mut info = Vec::new();

    for row in rows {
        info.push(parse_row(row)?);
    }

    Ok(info)
}

use heck::TitleCase;

fn parse_row(row: Vec<TextObject<'_>>) -> Result<MemberInfo, Error> {
    match row.as_slice() {
        [TextObject { text: surname, .. },
         TextObject { text: first_name, .. },
         TextObject { text: email, .. },
         TextObject { text: mobile, .. },
         _, _] =>
        {
            Ok(MemberInfo {
                surname: surname.to_title_case(),
                first_name: first_name.to_string(),
                email: email.to_string(),
                mobile: mobile.to_string(),
            })
        }
        other => Err(anyhow::anyhow!(
            "A row should have exactly 6 text fields, found {}",
            other.len()
        )),
    }
}

use anyhow::{Context, Error};
use pdf::file::File;

pub fn parse(pdf_blob: &[u8]) -> Result<ContactList, Error> {
    let pdf = File::from_data(pdf_blob)
        .context("Unable to parse the data as a PDF")?;

    let mut members = Vec::new();

    for (i, page) in pdf.pages().enumerate() {
        let page = page?;
        let members_on_page = parse_members_on_page(&page)
            .with_context(|| format!("Unable to parse the members on page {}", i + 1))?;

        members.extend(members_on_page);
    }

    Ok(ContactList { members })
}