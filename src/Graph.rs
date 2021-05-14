use serde_json::Value;
use std::collections::HashMap;
use std::collections::LinkedList;

trait GetProperties {
    fn startnode(&self) -> String;
    fn endnode(&self) -> String;
    fn linestring(&self) -> String;
    fn reference(&self) -> String;
    fn print(&self);
}

impl GetProperties for Value {
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
        println!("{}", serde_json::to_string_pretty(self).unwrap());
    }
}

trait CreateCoordinates {
    fn create_coordinates(&self) -> LinkedList<Coordinate>;
    fn create_startcoordinate(&self) -> Coordinate;
    fn create_endcoordinate(&self) -> Coordinate;
}

impl CreateCoordinates for String {
    fn create_coordinates(&self) -> LinkedList<Coordinate> {
        let i: usize;
        if self.starts_with("LINESTRING Z") {
            i = 14;
        }
        else {
            i = 11;
        }

        let actual_string = &self[i..self.len() - 1];
        let coordinatestrings: Vec<String> =
            actual_string.split(", ").map(|x| x.to_string()).collect();

        let mut coordinates: LinkedList<Coordinate> = LinkedList::new();

        for b in &coordinatestrings {
            let temp_coords = b.split(" ").map(|x| x.to_string()).collect::<Vec<String>>();

            let c = if temp_coords.len() == 3 {
                Coordinate::new(
                    String::from(temp_coords[0].as_str()),
                    String::from(temp_coords[1].as_str()),
                    String::from(temp_coords[2].as_str()),
                )
            }
            else {
                Coordinate::new(
                    String::from(temp_coords[0].as_str()),
                    String::from(temp_coords[1].as_str()),
                    String::from("N/A"),
                )
            };
            coordinates.push_back(c);
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

pub struct CustomGraph {}

impl CustomGraph {
    pub fn new() -> CustomGraph {
        CustomGraph {}
    }

    pub fn add_links(
        &mut self,
        json: Value,
        nodes: &mut HashMap<String, Node>,
        links: &mut HashMap<String, RoadLink>,
    ) {
        for b in json["objekter"].as_array().unwrap() {
            self.add_startnode(&b, nodes);
            self.add_endnode(&b, nodes);
            self.add_link(&b, links);
        }
    }

    fn add_startnode(&mut self, json: &Value, nodes: &mut HashMap<String, Node>) {
        self.add_node(
            nodes,
            json,
            Value::startnode,
            json.linestring().create_startcoordinate(),
        );
        nodes
            .get_mut(&Value::startnode(json))
            .unwrap()
            .add_outgoing(
                json["vegreferanse"]["kortform"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
    }

    fn add_endnode(&mut self, json: &Value, nodes: &mut HashMap<String, Node>) {
        self.add_node(
            nodes,
            json,
            Value::endnode,
            json.linestring().create_endcoordinate(),
        );
        nodes.get_mut(&Value::endnode(json)).unwrap().add_incoming(
            json["vegreferanse"]["kortform"]
                .as_str()
                .unwrap()
                .to_string(),
        );
    }

    fn add_node(
        &mut self,
        nodes: &mut HashMap<String, Node>,
        json: &Value,
        func: fn(&Value) -> String,
        coordinate: Coordinate,
    ) {
        if !nodes.contains_key(&func(json)) {
            nodes.insert(func(json), Node::new(String::from(func(json)), coordinate));
        }
    }

    fn add_link(&mut self, json: &Value, links: &mut HashMap<String, RoadLink>) {
        if !links.contains_key(&json.reference()) {
            links.insert(
                json.reference(),
                RoadLink::new(
                    json.reference(),
                    json.linestring(),
                    json.startnode(),
                    json.endnode(),
                ),
            );
        }
    }

    pub fn print(
        &self,
        nodes: &HashMap<String, Node>,
        links: &HashMap<String, RoadLink>,
    ) -> String {
        links
            .values()
            .map(|x| x.print(nodes, links))
            .collect::<String>()
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
            id: id,
            coordinate: coordinate,
        }
    }

    pub fn visit(&self, visited: &mut HashMap<String, String>) {
        visited.insert(self.id.clone(), self.id.clone());
    }

    fn add_incoming(&mut self, linkref: String) {
        self.incoming_links.push(linkref);
    }

    fn add_outgoing(&mut self, linkref: String) {
        self.outgoing_links.push(linkref);
    }

    pub fn print_links_to(&self, links: &HashMap<String, RoadLink>) -> String {
        format!(
            "incoming links: {}\noutgoing links: {}\n",
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

#[derive(Hash)]
pub struct RoadLink {
    id: String,
    start: String,
    end: String,
    coordinates: LinkedList<Coordinate>,
}

impl RoadLink {
    fn new(id: String, raw_linestring: String, start: String, end: String) -> RoadLink {
        RoadLink {
            id: id,
            coordinates: raw_linestring.create_coordinates(),
            start: start,
            end: end,
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
            "Link {}:\n\nStartnode: {}\n{}\nEndnode: {}\n{}\nCoordinates: {}\n\n",
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

impl PartialEq for RoadLink {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for RoadLink {}

#[derive(Hash)]
struct Coordinate {
    e: String,
    n: String,
    h: String,
}

impl Coordinate {
    fn new(e: String, n: String, h: String) -> Coordinate {
        Coordinate { e: e, n: n, h: h }
    }
}

impl PartialEq for Coordinate {
    fn eq(&self, other: &Self) -> bool {
        self.e == other.e && self.n == other.n && self.h == other.h
    }
}

impl std::fmt::Display for Coordinate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            if self.e.contains(".") {
                String::from(self.e.as_str())
            }
            else {
                String::from(self.e.as_str()) + "\t"
            },
            if self.n.contains(".") {
                String::from(self.n.as_str())
            }
            else {
                String::from(self.n.as_str()) + "\t"
            },
            if self.h.contains(".") {
                String::from(self.h.as_str())
            }
            else {
                String::from(self.h.as_str()) + "\t"
            },
        )
    }
}
