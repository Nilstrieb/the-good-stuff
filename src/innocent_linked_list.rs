pub struct Node<'a, 'n, T> {
    item: T,
    outer: Option<&'a mut Node<'a, 'n, T>>,
}

impl<'a, 'n, T> Node<'a, 'n, T> {
    pub fn new(item: T) -> Self {
        Self { item, outer: None }
    }

    pub fn push<R>(&mut self, item: T, with_func: impl FnOnce(&mut Self) -> R) -> R {
        let mut inner = Node {
            item,
            outer: Some(self),
        };
        with_func(&mut inner)
    }
}

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn push() {
        let mut list = Node::<u8>::new(0);

        inner(&mut list);

        fn inner(list: &mut Node<u8>) {
            list.push(1, |list| {});
        }
    }
}
