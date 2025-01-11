use new_macro::new;

#[derive(new)]
pub struct Dumby<T:Copy>{
	a:u8,
	b:T
}

impl<T: Copy> Dumby<T> where u8: From<T>{
	pub fn do_stuff(&self) -> u8{
		assert!(self.a!=0);
		let b:u8 = self.b.into();
		return self.a+b
	}
}

#[derive(new)]
pub struct MyUnit;

impl MyUnit{
	pub fn do_stuff(&self){
	}
}


#[derive(new)]
pub struct Dumby2<'a, T>(&'a T);

impl<T: Clone> Dumby2<'_, T>{
	pub fn do_stuff(&self) -> T{
		return self.0.clone()
	}
}


#[test]
fn check(){
	let _ = Dumby::<u16>::new(2,7);
	let _ = MyUnit::new();
	let _ = Dumby2::new(&2);
	println!("hello world");
} 

#[test]
fn test_arc(){
	let _ = Dumby::<u16>::new_arc(2,7);
	let _ = MyUnit::new_arc();
	let _ = Dumby2::new_arc(&2);
	println!("hello world");
} 

#[cfg(not(miri))] 
#[test]
fn test_fails() {
    let t = trybuild::TestCases::new();
    
    for entry in std::fs::read_dir("tests/fails").expect("Failed to read `tests/fails` directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.extension().map(|ext| ext == "rs").unwrap_or(false) {
            t.compile_fail(path.to_str().expect("Failed to convert path to string"));
        }
    }
}
