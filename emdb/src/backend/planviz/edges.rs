use crate::plan;
use super::PlanNode;
use dot;
#[derive(Clone)]
pub enum PlanEdge {
    DataFlow { data: plan::Key<plan::DataFlow>, flow_forward: bool, to_direction: bool },

    TableAccess { op: plan::Key<plan::Operator>,  table: plan::Key<plan::Table> },
    QueryReturn { op: plan::Key<plan::Operator>, query: plan::Key<plan::Query> },

    ModificationOrder { op: plan::Key<plan::Operator>, prev: plan::Key<plan::Operator> },
}

pub fn get_dataflow(self_key: plan::Key<plan::DataFlow>, df: &plan::DataFlow, flow_forward: bool, to_direction: bool, get_source: bool) -> PlanNode {
    let flow_comp = |from, to| {
        // NOTE: can be done with single if an xor, but clearer this way
        if get_source {
            if flow_forward {
                from
            } else {
                to
            }
        } else {
            if flow_forward {
                to
            } else {
                from
            }
        }
    };
    let self_node = PlanNode::Dataflow(self_key);
    if let plan::DataFlow::Conn { from, to, with } = df {
        if to_direction {
            flow_comp(self_node, PlanNode::Operator(*to))
        } else {
            flow_comp(PlanNode::Operator(*from), self_node)
        }
    } else {
        unreachable!("Only `DataFlow::Conn` edges should be present in dataflow")
    }
}

pub trait GetEdges: Sized {
    fn get_edges(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>);
}

impl GetEdges for plan::DataFlow {
    fn get_edges(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>) {
        if let plan::DataFlow::Conn { from, to, with } = self {
            edges.push(PlanEdge::DataFlow { data: self_key, flow_forward: true, to_direction: false });
            edges.push(PlanEdge::DataFlow { data: self_key, flow_forward: false, to_direction: false });
            edges.push(PlanEdge::DataFlow { data: self_key, flow_forward: true, to_direction: true });
            edges.push(PlanEdge::DataFlow { data: self_key, flow_forward: false, to_direction: true });
        } else {
            unreachable!("Only `DataFlow::Conn` edges should be present in dataflow")
        }
    }
}

impl GetEdges for plan::Operator {
    fn get_edges(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>) {
        let Self {query, kind} = self;
        match kind {
            plan::OperatorKind::Modify { modify_after, op } => {
                if let Some(after) = modify_after {
                    edges.push(PlanEdge::ModificationOrder { op: self_key, prev: *after });
                }
                edges.push(PlanEdge::TableAccess { op: self_key, table: op.get_table_access() });
            },
            plan::OperatorKind::Access { access_after, op } => {
                if let Some(after) = access_after {
                    edges.push(PlanEdge::ModificationOrder { op: self_key, prev: *after });
                }
                edges.push(PlanEdge::TableAccess { op: self_key, table: op.get_table_access() });
            },
            plan::OperatorKind::Flow(plan::FlowOperator::Return(_)) => {
                edges.push(PlanEdge::QueryReturn { op: self_key, query: *query });
            },
            _ => ()
        }
    }
}

#[enumtrait::store(get_access_trait)]
trait GetTableAccess {
    fn get_table_access(&self) -> plan::Key<plan::Table>;
}

impl GetTableAccess for plan::Update {
    fn get_table_access(&self) -> plan::Key<plan::Table> {
        self.table
    }
}

impl GetTableAccess for plan::Insert {
    fn get_table_access(&self) -> plan::Key<plan::Table> {
        self.table
    }
}

impl GetTableAccess for plan::Delete {
    fn get_table_access(&self) -> plan::Key<plan::Table> {
        self.table
    }
}

impl GetTableAccess for plan::GetUnique {
    fn get_table_access(&self) -> plan::Key<plan::Table> {
        self.table
    }
}

impl GetTableAccess for plan::Scan {
    fn get_table_access(&self) -> plan::Key<plan::Table> {
        self.table
    }
}

impl GetTableAccess for plan::DeRef {
    fn get_table_access(&self) -> plan::Key<plan::Table> {
        self.table
    }
}

#[enumtrait::impl_trait(get_access_trait for plan::modify_operator_enum)]
impl GetTableAccess for plan::ModifyOperator {}

#[enumtrait::impl_trait(get_access_trait for plan::access_operator_enum)]
impl GetTableAccess for plan::AccessOperator {}

// Wraps [`dot::Labeller`] to be implemented for graph nodes
trait StyleableEdge {
    fn label<'a>(&'a self) -> dot::LabelText<'a>;
    fn end_arrow(&self) -> dot::Arrow;
    fn start_arrow(&self) -> dot::Arrow;
    fn edge_style(&self) -> dot::Style;
    fn edge_color<'a>(&'a self) -> Option<dot::LabelText<'a>>;
}