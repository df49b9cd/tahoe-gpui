//! Connection data types for workflow graphs.
//!
//! Pure data structures representing connections between node ports.
//! No rendering — these are consumed by [`super::WorkflowCanvas`] to draw edges.

use super::edge::EdgeStyle;
use super::util::HandlePosition;

/// Identifies a specific port on a specific node.
#[derive(Debug, Clone, PartialEq)]
pub struct PortId {
    /// The node that owns the port.
    pub node_id: String,
    /// The port name on that node.
    pub port_name: String,
}

impl PortId {
    /// Create a new port identifier.
    pub fn new(node_id: impl Into<String>, port_name: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            port_name: port_name.into(),
        }
    }
}

/// A directed connection between two ports in the workflow graph.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Connection {
    /// Unique identifier for this connection.
    pub id: String,
    /// The source (output) port.
    pub source: PortId,
    /// The target (input) port.
    pub target: PortId,
    /// Optional label displayed on the edge.
    pub label: Option<String>,
    /// Visual style for this connection's edge.
    pub edge_style: EdgeStyle,
    /// Handle exit direction at the source node.
    pub source_position: HandlePosition,
    /// Handle exit direction at the target node.
    pub target_position: HandlePosition,
}

impl Connection {
    /// Create a new connection between two ports.
    pub fn new(id: impl Into<String>, source: PortId, target: PortId) -> Self {
        Self {
            id: id.into(),
            source,
            target,
            label: None,
            edge_style: EdgeStyle::default(),
            source_position: HandlePosition::Right,
            target_position: HandlePosition::Left,
        }
    }

    /// Set an optional label on the connection.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the visual edge style for this connection.
    pub fn edge_style(mut self, style: EdgeStyle) -> Self {
        self.edge_style = style;
        self
    }

    /// Set the handle exit direction at the source node.
    pub fn source_position(mut self, pos: HandlePosition) -> Self {
        self.source_position = pos;
        self
    }

    /// Set the handle exit direction at the target node.
    pub fn target_position(mut self, pos: HandlePosition) -> Self {
        self.target_position = pos;
        self
    }

    /// Validate that source and target are not on the same node.
    pub fn is_valid(&self) -> bool {
        self.source.node_id != self.target.node_id
    }
}

#[cfg(test)]
mod tests {
    use super::{Connection, HandlePosition, PortId};
    use core::prelude::v1::test;

    #[test]
    fn port_id_new() {
        let p = PortId::new("node1", "output");
        assert_eq!(p.node_id, "node1");
        assert_eq!(p.port_name, "output");
    }

    #[test]
    fn port_id_equality() {
        let a = PortId::new("n1", "p1");
        let b = PortId::new("n1", "p1");
        assert_eq!(a, b);
    }

    #[test]
    fn port_id_inequality() {
        let a = PortId::new("n1", "p1");
        let b = PortId::new("n1", "p2");
        assert_ne!(a, b);
    }

    #[test]
    fn connection_new() {
        let c = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n2", "in"));
        assert_eq!(c.id, "c1");
        assert!(c.label.is_none());
    }

    #[test]
    fn connection_with_label() {
        let c = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n2", "in"))
            .label("data flow");
        assert_eq!(c.label.unwrap(), "data flow");
    }

    #[test]
    fn connection_is_valid_different_nodes() {
        let c = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n2", "in"));
        assert!(c.is_valid());
    }

    #[test]
    fn connection_is_invalid_same_node() {
        let c = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n1", "in"));
        assert!(!c.is_valid());
    }

    #[test]
    fn port_id_clone() {
        let a = PortId::new("n1", "p1");
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn connection_default_handle_positions() {
        let c = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n2", "in"));
        assert_eq!(c.source_position, HandlePosition::Right);
        assert_eq!(c.target_position, HandlePosition::Left);
    }

    #[test]
    fn connection_builder_handle_positions() {
        let c = Connection::new("c1", PortId::new("n1", "out"), PortId::new("n2", "in"))
            .source_position(HandlePosition::Top)
            .target_position(HandlePosition::Bottom);
        assert_eq!(c.source_position, HandlePosition::Top);
        assert_eq!(c.target_position, HandlePosition::Bottom);
    }
}
