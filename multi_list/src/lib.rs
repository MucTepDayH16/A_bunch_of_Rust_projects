pub mod list;

#[cfg(test)]
mod list_test {
    use crate::list::MultiList;

    #[test]
    fn push_pop() {
        let mut master = MultiList::new();

        master.push(" heh!").push(" world!");

        let mut branch = master.branch();

        master.push("Hello,");
        branch.push(" cruel").push("Goodbye,");

        assert_eq!(branch.pop(), Some("Goodbye,"));
        assert_eq!(branch.pop(), Some(" cruel"));
        assert_ne!(branch.pop(), Some(" world!"));
        assert_eq!(branch.pop(), None);

        assert_eq!(master.pop(), Some("Hello,"));
        assert_eq!(master.pop(), Some(" world!"));
        assert_eq!(master.pop(), Some(" heh!"));
        assert_eq!(master.pop(), None);
    }

    use std::{
        cmp::max,
        ops::{Add, Mul},
    };

    #[test]
    fn evaluate_str() {
        let mut master = MultiList::new();
        let mut branch;
        master.push("ld!").push("wor");
        branch = master.branch();
        branch.push(" cruel").push("Goodbye,");
        master.push("o, ").push("Hell");

        let std_add = |eval: String, item: &&str| eval + *item;
        assert_eq!("Hello, world!", master.evaluate(std_add, "".to_string()));
    }

    #[test]
    fn evaluate_i32() {
        let array: Vec<i32> = (1..=4).collect();
        let mut master = MultiList::new();
        for i in array.iter() {
            master.push(*i);
        }

        assert_eq!(array.iter().sum::<i32>(), master.evaluate(i32::add, 0));
        assert_eq!(array.iter().product::<i32>(), master.evaluate(i32::mul, 1));
        assert_eq!(
            array.iter().max(),
            Some(master.evaluate(max::<&i32>, &i32::MIN))
        )
    }

    #[test]
    fn sub_list() {
        let mut master = MultiList::new();
        master.push("A").push("B").push("C").push("D");

        let mut iter = MultiList::from_to(master.find(&"C"), master.find(&"A"));

        assert_eq!(Some(&"C"), iter.next());
        assert_eq!(Some(&"B"), iter.next());
        assert_ne!(Some(&"A"), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn common_node() {
        let mut master = MultiList::new();
        master.push("A").push("B");

        let mut branch = master.branch();
        branch.push("C");

        master.push("D").push("E");

        let node_b = master.find(&"B");
        assert_eq!(node_b, branch.find(&"B"));
        assert_eq!(node_b, MultiList::common(&master, &branch));
    }
}
