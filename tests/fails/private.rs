mod m{
	use auto_new::new;

	#[derive(new)]
	#[new_visibility(/*private*/)]
	#[no_new_arc]//check this as well
	pub struct Dumby<'a, T>(&'a T);

	impl<T: Clone> Dumby<'_, T>{
		pub fn do_stuff(&self) -> T{
			return self.0.clone()
		}
	}
}


fn main(){
	let _ = m::Dumby::new(&3);
	let _ = m::Dumby::new_arc(&2);
	println!("hello world");
} 