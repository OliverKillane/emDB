use crate::plan;
use super::{PlanNode, GetFeature, DisplayConfig};
use dot;

#[derive(Clone, Debug)]
pub enum DataFlowKind {
    Stream,
    Single,
    Call
}
#[derive(Clone, Debug)]
pub struct DataFlow { 
    pub data: plan::Key<plan::DataFlow>,
    pub op: plan::Key<plan::Operator>,
    pub to_direction: bool, // either (false) from op -> data, or (true) data -> op
    pub flow: DataFlowKind
}

#[derive(Clone, Debug)]
pub struct TableAccess { 
    pub op: plan::Key<plan::Operator>,  
    pub table: plan::Key<plan::Table> 
}

#[derive(Clone, Debug)]
pub struct QueryReturn { 
    pub op: plan::Key<plan::Operator>, 
    pub query: plan::Key<plan::Query> 
}

#[derive(Clone, Debug)]
pub struct ModificationOrder { 
    pub op: plan::Key<plan::Operator>, 
    pub prev: plan::Key<plan::Operator> 
}

/// DataFlow -> fields type
#[derive(Clone, Debug)]
pub struct DataFlowToType {
    pub df: plan::Key<plan::DataFlow>,
    pub ty: plan::Key<plan::Record>
}

#[derive(Clone, Debug)]
pub struct RecordTypeToScalar {
    pub record: plan::Key<plan::Record>,
    pub scalar: plan::Key<plan::ScalarType>,
    pub field_name: String,
}

#[derive(Clone, Debug)]
pub struct ScalarToRecord {
    pub scalar: plan::Key<plan::ScalarType>,
    pub record: plan::Key<plan::Record>,
}

#[derive(Clone, Debug)]
pub struct ScalarToTable {
    pub scalar: plan::Key<plan::ScalarType>,
    pub table: plan::Key<plan::Table>,
}

#[derive(Clone, Debug)]
pub struct ScalarToScalar {
    pub from: plan::Key<plan::ScalarType>,
    pub to: plan::Key<plan::ScalarType>,
}

#[derive(Clone, Debug)]
pub struct RecordToRecord {
    pub from: plan::Key<plan::Record>,
    pub to: plan::Key<plan::Record>,
}

#[derive(Clone, Debug)]
pub struct TableToScalar {
    pub table: plan::Key<plan::Table>,
    pub field: String,
    pub to: plan::Key<plan::ScalarType>,
}

#[derive(Clone, Debug)]
pub struct QueryToScalar {
    pub query: plan::Key<plan::Query>,
    pub param: String,
    pub to: plan::Key<plan::ScalarType>,
}

#[enumtrait::quick_enum]
#[enumtrait::quick_from]
#[derive(Clone, Debug)]
#[enumtrait::store(plan_edge_enum)]
pub enum PlanEdge {
    DataFlow,
    TableAccess,
    QueryReturn,
    ModificationOrder,
    DataFlowToType,
    RecordTypeToScalar,
    ScalarToRecord,
    ScalarToTable,
    ScalarToScalar,
    RecordToRecord,
    TableToScalar,
    QueryToScalar
}

pub fn get_edges<'a>(lp: &'a plan::Plan, config: &'a DisplayConfig) -> dot::Edges<'a, PlanEdge> {
    // TODO: use coroutines/generators to remove the need for multiple iters over operators and dataflow.
    //       - coroutines are currently on nightly, but not stable
    //       - libraries like [remit](https://docs.rs/remit/latest/remit/) can be used, but I want to
    //         reduce dependencies
    let mut edges = Vec::new();
    GetFeature::get_all(&mut edges, &lp.dataflow, config);
    GetFeature::get_all(&mut edges, &lp.scalar_types, config);
    GetFeature::get_all(&mut edges, &lp.record_types, config);
    GetFeature::get_all(&mut edges, &lp.tables, config);
    GetFeature::get_all(&mut edges, &lp.operators, config);
    GetFeature::get_all(&mut edges, &lp.queries, config);
    edges.into()
}


impl GetFeature<PlanEdge> for plan::DataFlow {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>, config: &DisplayConfig) {
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

            if config.display_types {
                edges.push(DataFlowToType { df: self_key, ty: with.fields }.into())
            }
        } else {
            unreachable!("Only `DataFlow::Conn` edges should be present in dataflow")
        }
    }
}

impl GetFeature<PlanEdge> for plan::Operator {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>, config: &DisplayConfig) {
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
            plan::OperatorKind::Flow(plan::FlowOperator::Return(_) | plan::FlowOperator::Discard(_)) => {
                edges.push(QueryReturn { op: self_key, query: *query }.into());
            },
            _ => ()
        }
    }
}

impl GetFeature<PlanEdge> for plan::Record {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>, config: &DisplayConfig) {
        if config.display_types {
            match self {
                plan::ConcRef::Conc(c) => {
                    for (field_name, ty) in c.fields.iter() {
                        edges.push(RecordTypeToScalar { record: self_key, scalar: *ty, field_name: field_name.to_string() }.into());
                    }
                },
                plan::ConcRef::Ref(r) => {
                    edges.push(RecordToRecord {from: self_key, to: *r}.into())
                },
            }
        }
    }
}

impl GetFeature<PlanEdge> for plan::ScalarType {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>, config: &DisplayConfig) {
        if config.display_types {
            match self {
                plan::ConcRef::Ref(r) => edges.push(ScalarToScalar {from: self_key, to: *r}.into()),
                plan::ConcRef::Conc(c) => match c {
                    plan::ScalarTypeConc::TableRef(t) => edges.push(ScalarToTable {scalar: self_key, table: *t}.into()),
                    plan::ScalarTypeConc::Bag(r) | plan::ScalarTypeConc::Record(r) => edges.push(ScalarToRecord {scalar: self_key, record: *r}.into()),
                    plan::ScalarTypeConc::Rust(_) => (),
                },
            }
        }
    }
}

impl GetFeature<PlanEdge> for plan::Table {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>, config: &DisplayConfig) {
        if config.display_types {
            for (field_name, col) in self.columns.iter() {
                edges.push(TableToScalar { table: self_key, field: field_name.to_string(), to: col.data_type }.into());
            }
        }
    }
}

impl GetFeature<PlanEdge> for plan::Query {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanEdge>, config: &DisplayConfig) {
        if config.display_types {
            for (param, ty) in self.params.iter() {
                edges.push(QueryToScalar { query: self_key, param: param.to_string(), to: *ty }.into());
            }
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
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label("")
    }
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
            dot::LabelText::label("")
        } else {
            dot::LabelText::label("")
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
        } else if self.to_direction {
            PlanNode::Dataflow(self.data)
        } else {
            PlanNode::Operator(self.op)
        }
    }
}
impl EdgeStyle for TableAccess {
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
        Some(dot::LabelText::label("aqua"))
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
        Some(dot::LabelText::label("crimson"))
    }

    fn get_side(&self, source_side: bool) -> PlanNode {
        if source_side {
            PlanNode::Operator(self.prev)
        } else {
            PlanNode::Operator(self.op)
        }
    }
}

impl EdgeStyle for DataFlowToType {
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
        Some(dot::LabelText::label("black"))
    }

    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::Dataflow(self.df)
        } else {
            PlanNode::RecordType(self.ty)
        }
    }
}

impl EdgeStyle for RecordTypeToScalar {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label(self.field_name.clone())
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::RecordType(self.record)
        } else {
            PlanNode::ScalarType(self.scalar)
        }
    }
}

impl EdgeStyle for ScalarToRecord {
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::ScalarType(self.scalar)
        } else {
            PlanNode::RecordType(self.record)
        }
    }
}

impl EdgeStyle for ScalarToTable {
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::ScalarType(self.scalar)
        } else {
            PlanNode::Table(self.table)
        }
    }
}

impl EdgeStyle for RecordToRecord {
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::RecordType(self.from)
        } else {
            PlanNode::RecordType(self.to)
        }
    }
}
impl EdgeStyle for ScalarToScalar{
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::ScalarType(self.from)
        } else {
            PlanNode::ScalarType(self.to)
        }
    }
}

impl EdgeStyle for TableToScalar {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label(self.field.clone())
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::Table(self.table)
        } else {
            PlanNode::ScalarType(self.to)
        }
    }
}

impl EdgeStyle for QueryToScalar {
    fn label<'a>(&self) -> dot::LabelText<'a> {
        dot::LabelText::label(self.param.clone())
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
        Some(dot::LabelText::label("black"))
    }
    
    fn get_side(&self,source_side:bool) -> PlanNode {
        if source_side {
            PlanNode::Query(self.query)
        } else {
            PlanNode::ScalarType(self.to)
        }
    }
}
