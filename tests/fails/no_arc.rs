use auto_new::new;

#[derive(new)]
#[no_new_arc]
pub struct Dumby<'a, T>(&'a T);

impl<T: Clone> Dumby<'_, T>{
	pub fn do_stuff(&self) -> T{
		return self.0.clone()
	}
}

fn main(){
	let _ = Dumby::new(&3);
	let _ = Dumby::new_arc(&2);
	println!("hello world");
} 