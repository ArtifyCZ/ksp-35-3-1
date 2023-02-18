use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::io::stdin;
use anyhow::{format_err, Result};
use token_read::TokenReader;

#[derive(Copy, Clone, Debug)]
enum RoomType {
    Programmers,
    Managers,
    Empty,
}

#[derive(Debug)]
enum Vertex {
    Room(usize, RoomType),
    Hub(usize, Vec<(Vertex, bool)>),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Need {
    NoNeed,
    NeedOpen,
    NeedClosed,
}

impl Vertex {
    pub fn changes(&self, root: bool) -> (Need, usize) {
        match self {
            Vertex::Room(_id, room_type) => match room_type {
                RoomType::Programmers => (Need::NeedOpen, 0),
                RoomType::Managers => (Need::NeedClosed, 0),
                RoomType::Empty => (Need::NoNeed, 0),
            },
            Vertex::Hub(_id, neighbors) => {
                let mut need = Need::NoNeed;
                let mut open_changes = 0;
                let mut close_changes = 0;
                for (inner_need, inner_changes) in neighbors.iter()
                        .map(|(subtree, state)| match subtree.changes(false) {
                    (Need::NoNeed, _) => (Need::NoNeed, 0),
                    (Need::NeedClosed, inner) if root != false =>
                        (Need::NeedOpen, inner + if *state { 1 } else { 0 }),
                    (Need::NeedClosed, _) => (Need::NeedClosed, if *state { 1 } else { 0 }),
                    (Need::NeedOpen, inner) => (Need::NeedOpen, inner + if *state { 0 } else { 1 }),
                }) {
                    if need != Need::NeedOpen {
                        need = inner_need;
                    }

                    match inner_need {
                        Need::NoNeed => {},
                        Need::NeedOpen => {
                            open_changes += inner_changes;
                        },
                        Need::NeedClosed => {
                            close_changes += inner_changes;
                        },
                    }
                }
                match need {
                    Need::NoNeed => (Need::NoNeed, 0),
                    Need::NeedOpen => (Need::NeedOpen, open_changes + close_changes),
                    Need::NeedClosed => (Need::NeedClosed, 0),
                }
            }
        }
    }
}

struct Input {
    n: usize,
    m: usize,
    tree: Vertex,
}

fn get_or_insert<TK: 'static + Copy + Ord, TV: 'static + Default>(map: &mut BTreeMap<TK, TV>, key: TK) -> &mut TV {
    if !map.contains_key(&key) {
        map.insert(key, Default::default());
    }

    map.get_mut(&key).unwrap()
}

fn read_input() -> Result<Input> {
    let mut input = TokenReader::new(stdin().lock());

    let (n, m) = input.line()?;

    let edges: BTreeMap<usize, Vec<(usize, bool)>> = {
        let mut edges: BTreeMap<usize, Vec<(usize, bool)>> = BTreeMap::new();
        for line in input.take(n - 1) {
            let (a, b, s): (usize, usize, char) = line?; // first vertex, second vertex, edge state
            let s = s == 'O';

            get_or_insert(&mut edges, a).push((b, s));
            get_or_insert(&mut edges, b).push((a, s));
        }

        let mut e: BTreeMap<usize, Vec<(usize, bool)>> = BTreeMap::new();

        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();

        queue.push_back(1);

        while let Some(a) = queue.pop_front() {
            if seen.contains(&a) {
                continue;
            }

            seen.insert(a);

            for (b, s) in edges.get(&a).ok_or(format_err!("Foo"))? {
                if seen.contains(&b) {
                    continue;
                }

                get_or_insert(&mut e, a).push((*b, *s));
                queue.push_back(*b);
            }
        }

        e
    };

    let rooms: BTreeMap<usize, RoomType> = {
        let mut rooms = BTreeMap::new();

        for line in input.take(m) {
            let (a, t): (usize, char) = line?;
            let t = match t {
                'P' => RoomType::Programmers,
                'M' => RoomType::Managers,
                'E' => RoomType::Empty,
                _ => unreachable!(),
            };

            rooms.insert(a, t);
        }

        rooms
    };

    let tree: Vertex = {
        fn build_subtree(a: usize,
                         edges: &BTreeMap<usize, Vec<(usize, bool)>>,
                         rooms: &BTreeMap<usize, RoomType>,
                         depth: usize)
            -> Result<Vertex> {
            match rooms.get(&a) {
                Some(room) => Ok(Vertex::Room(a, *room)),
                None => {
                    let neighbors = edges.get(&a).ok_or(format_err!("Foo"))?
                        .iter().map(|(b, s)| Ok((build_subtree(*b, edges, rooms, depth + 1)?, *s)))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Vertex::Hub(a, neighbors))
                }
            }
        }

        build_subtree(1, &edges, &rooms, 0)
    }?;

    Ok(Input {
        n,
        m,
        tree,
    })
}

fn main() -> Result<()> {
    let Input {
        n: _n,
        m: _m,
        tree,
    } = read_input()?;

    println!("{}", match tree.changes(true) { (_, changes) => changes });

    Ok(())
}
