use crate::plan;
use super::PlanNode;
use dot;

#[derive(Clone)]
pub enum DataFlowKind {
    Stream,
    Single,
    Call
}
#[derive(Clone)]
pub struct DataFlow { 
    pub data: plan::Key<plan::DataFlow>,
    pub op: plan::Key<plan::Operator>,
    pub to_direction: bool, // either (false) from op -> data, or (true) data -> op
    pub flow: DataFlowKind
}

#[derive(Clone)]
pub struct TableAccess { 
    pub op: plan::Key<plan::Operator>,  
    pub table: plan::Key<plan::Table> 
}

#[derive(Clone)]
pub struct QueryReturn { 
    pub op: plan::Key<plan::Operator>, 
    pub query: plan::Key<plan::Query> 
}

#[derive(Clone)]
pub struct ModificationOrder { 
    pub op: plan::Key<plan::Operator>, 
    pub prev: plan::Key<plan::Operator> 
}


#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[derive(Clone)]
#[enumtrait::store(plan_edge_enum)]
pub enum PlanEdge {
    DataFlow,
    TableAccess,
    QueryReturn,
    ModificationOrder
}

pub trait GetEdges: Sized {
    fn get_edges(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>);
}

impl GetEdges for plan::DataFlow {
    fn get_edges(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>) {
        if let plan::DataFlow::Conn { from, to, with } = self {
            let stream = if with.stream {
                DataFlowKind::Stream
            } else {
                DataFlowKind::Single
            };
            edges.push(DataFlow { data: self_key, op: *from, to_direction: true, flow: stream.clone()  }.into());
            edges.push(DataFlow { data: self_key, op: *from, to_direction: false, flow: DataFlowKind::Call  }.into());
            edges.push(DataFlow { data: self_key, op: *to, to_direction: false, flow: stream  }.into());
            edges.push(DataFlow { data: self_key, op: *to, to_direction: true, flow: DataFlowKind::Call  }.into());
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
                    edges.push(ModificationOrder { op: self_key, prev: *after }.into());
                }
                edges.push(TableAccess { op: self_key, table: op.get_table_access() }.into());
            },
            plan::OperatorKind::Access { access_after, op } => {
                if let Some(after) = access_after {
                    edges.push(ModificationOrder { op: self_key, prev: *after }.into());
                }
                edges.push(TableAccess { op: self_key, table: op.get_table_access() }.into());
            },
            plan::OperatorKind::Flow(plan::FlowOperator::Return(_)) => {
                edges.push(QueryReturn { op: self_key, query: *query }.into());
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
#[enumtrait::store(edge_style_trait)]
pub trait EdgeStyle {
    fn label<'a>(&self) -> dot::LabelText<'a>;
    fn end_arrow(&self) -> dot::Arrow;
    fn start_arrow(&self) -> dot::Arrow;
    fn edge_style(&self) -> dot::Style;
    fn edge_color<'a>(&self) -> Option<dot::LabelText<'a>>;
    fn get_side(&self, source_side: bool) -> PlanNode;
}

#[enumtrait::impl_trait(edge_style_trait for plan_edge_enum)]
impl EdgeStyle for PlanEdge {}

impl EdgeStyle for DataFlow {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        if let DataFlowKind::Call = self.flow {
            dot::LabelText::label("call")
        } else {
            dot::LabelText::label("data")
        }
    }

    fn end_arrow(&self) -> dot::Arrow {
        match self.flow {
            DataFlowKind::Stream | DataFlowKind::Single => dot::Arrow::from_arrow(dot::ArrowShape::normal()),
            DataFlowKind::Call => dot::Arrow::from_arrow(dot::ArrowShape::vee()),
        }
    }

    fn start_arrow(&self) -> dot::Arrow {
        dot::Arrow::none()
    }

    fn edge_style(&self) -> dot::Style {
        match self.flow {
            DataFlowKind::Stream => dot::Style::Bold,
            DataFlowKind::Single => dot::Style::None,
            DataFlowKind::Call => dot::Style::Dashed,
        }
    }

    fn edge_color<'a>(&self) -> Option<dot::LabelText<'a>> {
        match self.flow {
            DataFlowKind::Stream | DataFlowKind::Single => Some(dot::LabelText::label("darkorchid1")),
            DataFlowKind::Call => Some(dot::LabelText::label("darksalmon"))
        }
    }

    fn get_side(&self, source_side: bool) -> PlanNode {
        if source_side {
            if self.to_direction {
                PlanNode::Operator(self.op)
            } else {
                PlanNode::Dataflow(self.data)
            }
        } else {
            if self.to_direction {
                PlanNode::Dataflow(self.data)
            } else {
                PlanNode::Operator(self.op)
            }
        }
    }
}
impl EdgeStyle for TableAccess {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label("access")
    }

    fn end_arrow(&self) -> dot::Arrow {
        dot::Arrow::normal()
    }

    fn start_arrow(&self) -> dot::Arrow {
        dot::Arrow::none()
    }

    fn edge_style(&self) -> dot::Style {
        dot::Style::None
    }

    fn edge_color<'a>(&self) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkorchid1"))
    }

    fn get_side(&self, source_side: bool) -> PlanNode {
        if source_side {
            PlanNode::Operator(self.op)
        } else {
            PlanNode::Table(self.table)
        }
    }
}
impl EdgeStyle for QueryReturn {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label("return")
    }

    fn end_arrow(&self) -> dot::Arrow {
        dot::Arrow::normal()
    }

    fn start_arrow(&self) -> dot::Arrow {
        dot::Arrow::none()
    }

    fn edge_style(&self) -> dot::Style {
        dot::Style::None
    }

    fn edge_color<'a>(&self) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkorchid1"))
    }

    fn get_side(&self, source_side: bool) -> PlanNode {
        if source_side {
            PlanNode::Operator(self.op)
        } else {
            PlanNode::Query(self.query)
        }
    }
}
impl EdgeStyle for ModificationOrder {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label("modify after")
    }

    fn end_arrow(&self) -> dot::Arrow {
        dot::Arrow::from_arrow(dot::ArrowShape::vee())
    }

    fn start_arrow(&self) -> dot::Arrow {
        dot::Arrow::none()
    }

    fn edge_style(&self) -> dot::Style {
        dot::Style::Dashed
    }

    fn edge_color<'a>(&self) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darksalmon"))
    }

    fn get_side(&self, source_side: bool) -> PlanNode {
        if source_side {
            PlanNode::Operator(self.op)
        } else {
            PlanNode::Operator(self.prev)
        }
    }
}
