use auto_new::new;

#[derive(new)]
enum Dumby{
	A(u8),
	B(bool)
}