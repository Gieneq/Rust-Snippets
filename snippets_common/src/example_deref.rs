use std::ops::Deref;

struct A {
    value: u32,
}

impl Deref for A {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

fn unpack_refs<T, U>(value: &T) -> U
where 
    T: Deref<Target = U>,
    U: Copy
{
    *value.deref()
}

fn sum_refs<A, B, U>(left: &A, right: &B) -> U
where 
    A: Deref<Target = U>,
    B: Deref<Target = U>,
    U: std::ops::Add<Output = U> + Copy
{
    *left.deref() + *right.deref()
}

#[test]
fn test_sth() {
    let a = A {value: 3};
    let r: u32 = unpack_refs(&a);
    
    let b = Box::new(3u32);
    let x: u32 = unpack_refs(&b);

    let c = sum_refs(&a, &b);
}


// #[test]
// fn test_sth() {
//     let a = 2u32;
//     let b = 3u32;
//     sum_refs(&a, &b); // ✅
    
//     let a = Box::new(2u32);
//     let b = Box::new(3u32);
//     sum_refs(&a, &b); // ✅ thanks to deref coercion
    
// }