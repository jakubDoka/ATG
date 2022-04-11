pub struct Graph {
    pub hints: Vec<usize>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;

        let line_count = content.lines().count();
        let mut hints = Vec::with_capacity(line_count);
        let mut edges = Vec::with_capacity(line_count);
        let mut previous = 1;
        hints.push(0);
        for (i, line) in content.lines().enumerate() {
            let mut parts = line.split_whitespace().map(|s| s.parse::<usize>());
            let (Some(Ok(from)), Some(Ok(to)), Some(Ok(weight))) = (parts.next(), parts.next(), parts.next()) else {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid data"));
            };

            while previous != from {
                hints.push(i);
                previous += 1;
            }

            edges.push(Edge { from, to, weight });
        }
        hints.push(line_count as usize);

        Ok(Graph { hints, edges })
    }

    pub fn children(&self, node: usize) -> &[Edge] {
        &self.edges[self.hints[node - 1]..self.hints[node]]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: usize,
}
