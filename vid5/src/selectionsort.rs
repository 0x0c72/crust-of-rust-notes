use super::Sorter;

pub struct SelectionSort;

impl Sorter for SelectionSort {
    fn sort<T>(&self, slice: &mut [T])
    where
        T: Ord,
    {
        // can't assume first element is sorted, needs to be smallest element
        for unsorted in 0..slice.len() {
            // function way
            let smallest_in_rest = slice[unsorted..]
                .iter()
                .enumerate()
                .min_by_key(|&(_, v)| v) // pull out just the value
                .map(|(i, _)| unsorted + i)
                .expect("!slice is non-empty"); // we know it'll never be empty
            // or
            // explicit way
            // let mut smallest_in_rest_2 = unsorted;
            // // check for smallest element in remainder and insert it where it goes
            // for i in (unsorted + 1)..slice.len() {
            //     if slice[i] < slice[smallest_in_rest_2] {
            //         smallest_in_rest_2 = i;
            //     }
            // }
            // assert_eq!(smallest_in_rest, smallest_in_rest_2);

            if unsorted != smallest_in_rest {
                slice.swap(unsorted, smallest_in_rest);
            }
        }
    }
}

#[test]
fn it_works() {
    let mut things = vec![4, 2, 5, 3, 1];
    SelectionSort.sort(&mut things);
    assert_eq!(things, vec![1, 2, 3, 4, 5]);
}
