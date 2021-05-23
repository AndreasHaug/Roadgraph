use rand::Rng;
use serde_json::{to_string_pretty, Value};
use std::collections::HashMap;
use std::collections::LinkedList;

trait RoadProperties {
    fn startnode(&self) -> String;
    fn endnode(&self) -> String;
    fn linestring(&self) -> String;
    fn reference(&self) -> String;
    fn print(&self);
}

impl RoadProperties for Value {
    fn startnode(&self) -> String {
        String::from(self["startnode"].as_str().unwrap())
    }

    fn endnode(&self) -> String {
        String::from(self["sluttnode"].as_str().unwrap())
    }

    fn linestring(&self) -> String {
        String::from(self["geometri"]["wkt"].as_str().unwrap())
    }

    fn reference(&self) -> String {
        String::from(self["vegreferanse"]["kortform"].as_str().unwrap())
    }

    fn print(&self) {
        println!("{}", to_string_pretty(self).unwrap());
    }
}

trait CreateCoordinates {
    fn create_coordinates(&self) -> LinkedList<Coordinate>;
    fn create_startcoordinate(&self) -> Coordinate;
    fn create_endcoordinate(&self) -> Coordinate;
}

impl CreateCoordinates for String {
    fn create_coordinates(&self) -> LinkedList<Coordinate> {
        let i: usize = match self.starts_with("LINESTRING Z") {
            true => 14,
            _ => 11,
        };

        let actual_string = &self[i..self.len() - 1];
        let coordinatestrings: Vec<String> =
            actual_string.split(", ").map(|x| x.to_string()).collect();

        let mut coordinates: LinkedList<Coordinate> = LinkedList::new();

        for b in &coordinatestrings {
            let temp_coords = b.split(' ').map(|x| x.to_string()).collect::<Vec<String>>();

            coordinates.push_back(Coordinate::new(
                String::from(temp_coords[0].as_str()),
                String::from(temp_coords[1].as_str()),
                match temp_coords.len() {
                    3 => String::from(temp_coords[2].as_str()),
                    _ => String::from("N/A"),
                },
            ));
        }

        coordinates
    }

    fn create_startcoordinate(&self) -> Coordinate {
        match self.create_coordinates().pop_front() {
            Some(s) => s,
            None => panic!("Could not get start coordinate from linestring"),
        }
    }

    fn create_endcoordinate(&self) -> Coordinate {
        match self.create_coordinates().pop_back() {
            Some(s) => s,
            None => panic!("Could not get end coordinate from linestring"),
        }
    }
}

pub struct Graph {
    nodes: HashMap<String, Node>,
    links: HashMap<String, RoadLink>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            nodes: HashMap::new(),
            links: HashMap::new(),
        }
    }

    pub fn add_links(&mut self, json: Value) {
        for b in json["objekter"].as_array().unwrap() {
            self.add_startnode(&b);
            self.add_endnode(&b);
            self.add_link(&b);
        }
    }

    fn add_startnode(&mut self, json: &Value) {
        self.add_node(
            json,
            Value::startnode,
            json.linestring().create_startcoordinate(),
        );
        self.nodes
            .get_mut(&Value::startnode(json))
            .unwrap()
            .add_outgoing(
                json["vegreferanse"]["kortform"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
    }

    fn add_endnode(&mut self, json: &Value) {
        self.add_node(
            json,
            Value::endnode,
            json.linestring().create_endcoordinate(),
        );
        self.nodes
            .get_mut(&Value::endnode(json))
            .unwrap()
            .add_incoming(
                json["vegreferanse"]["kortform"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
    }

    fn add_node(&mut self, json: &Value, func: fn(&Value) -> String, coordinate: Coordinate) {
        self.nodes
            .entry(func(json))
            .or_insert_with(|| Node::new(func(json), coordinate));
    }

    fn add_link(&mut self, json: &Value) {
        self.links
            .entry(json.reference())
            .or_insert_with(|| -> RoadLink {
                RoadLink::new(
                    json.reference(),
                    json.linestring(),
                    json.startnode(),
                    json.endnode(),
                )
            });
    }

    pub fn print(&self) -> String {
        self.links
            .values()
            .map(|x| x.print(&self.nodes, &self.links))
            .collect::<String>()
    }

    pub fn breadth_first(&self) {
        let mut visited = HashMap::new();
        let mut queue: LinkedList<(String, String)> = LinkedList::new();

        self.nodes
            .values()
            .nth(rand::thread_rng().gen_range(0..self.nodes.len() - 1))
            .unwrap()
            .explore_links(&mut visited, &self.links, &mut queue);

        while let Some(out) = queue.pop_front() {
            let link = self.links.get(&out.0).unwrap();
            let outnode;

            if out.1 == "start" {
                outnode = self.nodes.get(&link.start).unwrap();
                println!(
                    "{:<10}\t<------    \t{:<30}\t<------    \t{}",
                    link.end, out.0, link.start,
                );
            }
            else {
                outnode = self.nodes.get(&link.end).unwrap();
                println!(
                    "{:<10}\t------> \t{:<30}\t------> \t{}",
                    link.start, out.0, link.end
                );
            }
            outnode.explore_links(&mut visited, &self.links, &mut queue);
        }
    }
}

pub struct Node {
    outgoing_links: Vec<String>,
    incoming_links: Vec<String>,
    id: String,
    coordinate: Coordinate,
}

impl Node {
    fn new(id: String, coordinate: Coordinate) -> Node {
        Node {
            outgoing_links: Vec::new(),
            incoming_links: Vec::new(),
            id,
            coordinate,
        }
    }

    pub fn visit(&self, visited: &mut HashMap<String, String>) {
        visited.insert(self.id.clone(), self.id.clone());
    }

    pub fn explore_links(
        &self,
        visited: &mut HashMap<String, String>,
        links: &HashMap<String, RoadLink>,
        queue: &mut LinkedList<(String, String)>,
    ) {
        if !visited.contains_key(&self.id) {
            visited.insert(self.id.to_string(), self.id.to_string());
        }
        for a in &self.outgoing_links {
            let visited_id = (links.get(a).unwrap().end).to_string();

            if !visited.contains_key(&visited_id) {
                visited.insert(visited_id.to_string(), visited_id.to_string());
                queue.push_back((a.to_string(), "end".to_string()));
            }
        }
        for a in &self.incoming_links {
            let visited_id = (links.get(a).unwrap().start).to_string();

            if !visited.contains_key(&visited_id) {
                visited.insert(visited_id.to_string(), visited_id.to_string());
                queue.push_back((a.to_string(), "start".to_string()));
            }
        }
    }

    fn add_incoming(&mut self, linkref: String) {
        self.incoming_links.push(linkref);
    }

    fn add_outgoing(&mut self, linkref: String) {
        self.outgoing_links.push(linkref);
    }

    pub fn print_links_to(&self, links: &HashMap<String, RoadLink>) -> String {
        format!(
            "Incoming links: {}\nOutgoing links: {}\n",
            self.print_incoming_links(links),
            self.print_outgoing_links(links)
        )
    }

    pub fn print_outgoing_links(&self, links: &HashMap<String, RoadLink>) -> String {
        self.outgoing_links
            .iter()
            .map(|x| links.get(x).unwrap().get_id() + ", ")
            .collect::<String>()
    }

    pub fn print_incoming_links(&self, links: &HashMap<String, RoadLink>) -> String {
        self.incoming_links
            .iter()
            .map(|x| links.get(x).unwrap().get_id() + ", ")
            .collect::<String>()
    }
}

pub struct RoadLink {
    pub id: String,
    start: String,
    end: String,
    coordinates: LinkedList<Coordinate>,
}

impl RoadLink {
    fn new(id: String, raw_linestring: String, start: String, end: String) -> RoadLink {
        RoadLink {
            id,
            coordinates: raw_linestring.create_coordinates(),
            start,
            end,
        }
    }

    fn add_coordinate(&mut self, c: Coordinate) {
        self.coordinates.push_back(c);
    }

    fn print_coordinates(&self) -> String {
        self.coordinates
            .iter()
            .map(|x| x.to_string() + "\n")
            .collect::<String>()
            + "\n"
    }

    fn print_coordinates_reversed(&self) -> String {
        self.coordinates
            .iter()
            .rev()
            .map(|x| x.to_string() + "\n")
            .collect::<String>()
            + "\n"
    }

    pub fn print(
        &self,
        nodes: &HashMap<String, Node>,
        links: &HashMap<String, RoadLink>,
    ) -> String {
        format!(
            "Link {}:\n\nStartnode: {}\n{}\nEndnode: {}\n{}\nCoordinates:\n{}\n\n",
            self.id,
            self.start,
            nodes.get(&self.start).unwrap().print_links_to(links),
            self.end,
            nodes.get(&self.end).unwrap().print_links_to(links),
            self.print_coordinates(),
        )
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }
}

struct Coordinate {
    e: String,
    n: String,
    h: String,
}

impl Coordinate {
    fn new(e: String, n: String, h: String) -> Coordinate {
        Coordinate { e, n, h }
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<16}\t{:<16}\t{}",
            self.e.as_str(),
            self.n.as_str(),
            self.h.as_str()
        )
    }
}
