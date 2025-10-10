fn main() {
    // Variables and basic types
    let name = "Alice";
    let age = 30;
    let height: f64 = 5.7; // explicit type annotation
    
    println!("Hello, {}! You are {} years old.", name, age);
    
    // Mutable variables
    let mut counter = 0;
    counter += 1;
    println!("Counter: {}", counter);
    
    // Strings
    let mut greeting = String::from("Welcome to Rust");
    greeting.push_str("!");
    println!("{}", greeting);
    
    // Arrays and iteration
    let numbers = [1, 2, 3, 4, 5];
    println!("Sum of numbers:");
    let sum = sum_array(&numbers);
    println!("Sum: {}", sum);
    
    // Ownership and borrowing
    let s1 = String::from("hello");
    let s2 = &s1; // borrow s1
    println!("s1: {}, s2: {}", s1, s2); // both can be used
    
    // Pattern matching
    let result = divide(10, 2);
    match result {
        Ok(value) => println!("Result: {}", value),
        Err(msg) => println!("Error: {}", msg),
    }
    
    // Structs
    let person = Person {
        name: String::from("Bob"),
        age: 25,
    };
    person.introduce();
}

// Function that borrows an array
fn sum_array(arr: &[i32]) -> i32 {
    let mut total = 0;
    for &num in arr {
        total += num;
    }
    total
}

// Function returning Result type (for error handling)
fn divide(a: i32, b: i32) -> Result<i32, &'static str> {
    if b == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(a / b)
    }
}

// Struct definition
struct Person {
    name: String,
    age: u32,
}

// Implementation block
impl Person {
    fn introduce(&self) {
        println!("My name is {} and I'm {} years old", self.name, self.age);
    }
}