use super::Sorter;

pub struct QuickSort;

fn quicksort<T: Ord>(slice: &mut [T]) {
    match slice.len() {
        0 | 1 => return,
        2 => {
            if slice[0] > slice[1] {
                slice.swap(0, 1);
            }
            return;
        }
        _ => {}
    }


    let (pivot, rest) = slice.split_first_mut().expect("!slice is non-empty");
    let mut left = 0;
    let mut right = rest.len() - 1;
    // consider boundary conditions (what happens are start + finish)
    while left <= right {
        if &rest[left] <= pivot {
            // already on correct side
            left += 1;
        } else if &rest[right] > pivot {
            // right already on correct side
            // avoid unnecessary swaps back and forth)
            if right == 0 {
                break;
            }
            right -= 1;
        } else {
            // left holds a right and right holds a left, swap them
            // move this element to the right side
            rest.swap(left, right);
            left += 1;
            if right == 0 {
                break;
            }
            right -= 1;
        }
    }

    // re-align left to account for the pivot at 1
    let left = left + 1;
    
    // place the pivot at its final location
    slice.swap(0, left - 1);
    
    let (left, right) = slice.split_at_mut(left - 1); // split at the pivot
    assert!(left.last() <= right.first());
    quicksort(left);
    quicksort(&mut right[1..]);
}

impl Sorter for QuickSort {
    fn sort<T>(&self, slice: &mut [T])
    where
        T: Ord,
    {
        quicksort(slice);
    }
}

#[test]
fn it_works() {
    let mut things = vec![4, 2, 5, 3, 1, 6, 7, 8, 9];
    QuickSort.sort(&mut things);
    assert_eq!(things, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
}
