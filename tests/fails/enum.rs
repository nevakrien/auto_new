use new_macro::new;

#[derive(new)]
enum Dumby{
	A(u8),
	B(bool)
}