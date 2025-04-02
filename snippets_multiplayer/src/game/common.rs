pub type Vector2F = Vector2X<f32>;
pub type Vector2U = Vector2X<u32>;
pub type Vector2I = Vector2X<i32>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Vector2X<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2X<T> 
where 
    T: Default
{
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: T::default(), y: T::default() }
    }
}

impl<T> std::ops::Add for Vector2X<T> 
where 
    T: std::ops::Add<Output = T>
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x, 
            y: self.y + rhs.y
        }
        
    }
}

impl<T> std::ops::AddAssign for Vector2X<T> 
where
    T: std::ops::AddAssign
{
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Neg for Vector2X<f32> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::ops::Neg for Vector2X<i32> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> std::ops::Mul<T> for Vector2X<T> 
where 
    T: std::ops::Mul<Output = T> + Copy
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl From<Vector2X<f32>> for Vector2X<u32> {
    fn from(value: Vector2X<f32>) -> Self {
        Self { x: value.x as u32, y: value.y as u32 }
    }
}

impl From<Vector2X<u32>> for Vector2X<f32> {
    fn from(value: Vector2X<u32>) -> Self {
        Self { x: value.x as f32, y: value.y as f32 }
    }
}

#[test]
fn test_vector_creation() {
    let v1 = Vector2X::<f32>::new(1.0, 2.0);
    assert_eq!(v1.x, 1.0);
    assert_eq!(v1.y, 2.0);
}

#[test]
fn test_vector_add() {
    let v1 = Vector2X::<u32>::new(1, 2);
    let v2 = Vector2X::<u32>::new(10, 20);
    let v3 = v1 + v2;
    assert_eq!(v3.x, v1.x + v2.x);
    assert_eq!(v3.y, v1.y + v2.y);
}

#[test]
fn test_vector_add_assign() {
    let v1 = Vector2X::<u32>::new(1, 2);
    let mut v2 = Vector2X::<u32>::new(10, 20);
    v2 += v1;
    assert_eq!(v2.x, 11);
    assert_eq!(v2.y, 22);
}

#[test]
fn test_vector_negation() {
    let v1 = Vector2X::<i32>::new(1, 2);
    let v1_neg = -v1;
    assert_eq!(v1_neg.x, -v1.x);
    assert_eq!(v1_neg.y, -v1.y);
}

#[test]
fn test_vector_mul_scalar() {
    let v1 = Vector2X::<i32>::new(1, 2);
    let scalar = 5;
    let v1_multiplied = v1 * scalar;
    assert_eq!(v1_multiplied.x, v1.x * scalar);
    assert_eq!(v1_multiplied.y, v1.y * scalar);
}

#[test]
fn test_vector_casting() {
    let v1 = Vector2X::<f32>::new(1.2, 2.6);
    let v1_cast_u32 =  Vector2X::<u32>::from(v1);
    assert_eq!(v1_cast_u32.x, 1);
    assert_eq!(v1_cast_u32.y, 2);
}