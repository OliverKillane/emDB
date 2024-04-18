#![allow(clippy::useless_format)] // TODO: so clippy will pass while the nodes are not labelled
use crate::plan;
use super::{DisplayConfig, GetFeature};
use dot;
use quote::ToTokens;

#[derive(Clone, Debug)]
pub enum PlanNode {
    Table(plan::Key<plan::Table>),
    Operator(plan::Key<plan::Operator>),
    Dataflow(plan::Key<plan::DataFlow>),
    Query(plan::Key<plan::Query>),
    RecordType(plan::Key<plan::RecordType>),
    ScalarType(plan::Key<plan::ScalarType>),
    Context(plan::Key<plan::Context>),
}

// NOTE: complex but eliminates boilerplate
//       - match by query type, then apply the method
//       - variable names need to be passed (hygenic macro)
macro_rules! node_call {
    (match $self_id:ident , $node_id:ident -> $it:ident, $key:ident => $($tk:tt)*) => {
        match $node_id {
            PlanNode::Table($key) => {let $it = $self_id.plan.get_table(*$key); $($tk)* },
            PlanNode::Operator($key) => {let $it = $self_id.plan.get_operator(*$key); $($tk)* },
            PlanNode::Dataflow($key) => {let $it = $self_id.plan.get_dataflow(*$key); $($tk)* },
            PlanNode::Query($key) => {let $it = $self_id.plan.get_query(*$key); $($tk)* },
            PlanNode::RecordType($key) => {let $it = $self_id.plan.record_types.get(*$key).unwrap(); $($tk)* },
            PlanNode::ScalarType($key) => {let $it = $self_id.plan.scalar_types.get(*$key).unwrap(); $($tk)* },
            PlanNode::Context($key) => {let $it = $self_id.plan.get_context(*$key); $($tk)* },
        }
    }
}
pub(crate) use node_call;

impl GetFeature<PlanNode> for plan::DataFlow {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        edges.push(PlanNode::Dataflow(self_key));
    }
}
impl GetFeature<PlanNode> for plan::Table {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        edges.push(PlanNode::Table(self_key));
    }
}
impl GetFeature<PlanNode> for plan::Query {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        edges.push(PlanNode::Query(self_key));
    }
}
impl GetFeature<PlanNode> for plan::Context {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        edges.push(PlanNode::Context(self_key));
    }
}
impl GetFeature<PlanNode> for plan::Operator {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        edges.push(PlanNode::Operator(self_key));
    }
}
impl GetFeature<PlanNode> for plan::RecordType {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        if config.display_types {edges.push(PlanNode::RecordType(self_key))};
    }
}
impl GetFeature<PlanNode> for plan::ScalarType {
    fn get_features(&self, self_key: plan::Key<Self>, edges: &mut Vec<PlanNode>, config: &DisplayConfig) {
        if config.display_types {edges.push(PlanNode::ScalarType(self_key))};
    }
}

pub fn get_nodes<'a>(lp: &'a plan::Plan, config: &'a DisplayConfig) -> dot::Nodes<'a, PlanNode> {
    let mut nodes = Vec::new();

    GetFeature::get_all(&mut nodes, &lp.queries, config);
    GetFeature::get_all(&mut nodes, &lp.contexts, config);
    GetFeature::get_all(&mut nodes, &lp.tables, config);
    GetFeature::get_all(&mut nodes, &lp.operators, config);
    GetFeature::get_all(&mut nodes, &lp.dataflow, config);
    GetFeature::get_all(&mut nodes, &lp.scalar_types, config);
    GetFeature::get_all(&mut nodes, &lp.record_types, config);
    nodes.into()
}


// Wraps [`dot::Labeller`] to be implemented for graph nodes
pub trait StyleableNode: Sized {
    const ID_PREFIX: &'static str;
    fn id(&self, self_key: plan::Key<Self>) -> dot::Id<'_> {
        dot::Id::new(format!("{}_{}", Self::ID_PREFIX, self_key.arr_idx() )).unwrap()
    }

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>>;
    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a>;
    fn style(&self, plan: &plan::Plan) -> dot::Style;
    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>>;
}

impl StyleableNode for plan::RecordType {
    const ID_PREFIX: &'static str = "recordtype";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(match self {
            plan::ConcRef::Conc(_) => dot::LabelText::label("circle"),
            plan::ConcRef::Ref(_) => dot::LabelText::label("point"),
        })
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        match self {
            plan::ConcRef::Conc(_) => dot::LabelText::label("{ .. }"),
            plan::ConcRef::Ref(_) => dot::LabelText::label(""),
        } 
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        dot::Style::None
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkslategray1"))
    }
}

impl StyleableNode for plan::ScalarType {
    const ID_PREFIX: &'static str = "scalartype";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(match self {
            plan::ConcRef::Conc(_) => dot::LabelText::label("circle"),
            plan::ConcRef::Ref(_) => dot::LabelText::label("point"),
        })
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        match self {
            plan::ConcRef::Conc(c) => dot::LabelText::label(match c {
                plan::ScalarTypeConc::TableRef(r) => "ref".to_owned(),
                plan::ScalarTypeConc::Bag(_) => "bag".to_owned(),
                plan::ScalarTypeConc::Record(_) => "rec".to_owned(),
                plan::ScalarTypeConc::Rust(t) => format!("{}", t.to_token_stream()),
            }),
            plan::ConcRef::Ref(_) => dot::LabelText::label(""),
        }
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        dot::Style::None
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkslategray1"))
    }
}

impl StyleableNode for plan::Table {
    const ID_PREFIX: &'static str = "table";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("box3d"))
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        dot::LabelText::label(self.name.to_string())
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        dot::Style::Bold
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkslategray1"))
    }
}


impl StyleableNode for plan::Operator {
    const ID_PREFIX: &'static str = "operator";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("box"))
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        dot::LabelText::label(self.description(plan))
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        dot::Style::Bold
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("chartreuse1"))
    }
}

impl StyleableNode for plan::DataFlow {
    const ID_PREFIX: &'static str = "dataflow";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("ellipse"))
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        
        let plan::DataFlowConn {
            from,
            to,
            with
        } = self.get_conn();

        // NOTE: we opt not to show the type information here, as that would 
        //       require a graph traversal.
        //       Planviz is for debugging, if the type graph was cyclical 
        //       (bug) this would crash the planviz backend
        dot::LabelText::label(if with.stream { "stream" } else {"single"} )
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        let plan::DataFlowConn {
            from,
            to,
            with
        } = self.get_conn();
        if with.stream {
            dot::Style::Bold
        } else {
            dot::Style::None
        }
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkorchid1"))
    }
}

impl StyleableNode for plan::Query {
    const ID_PREFIX: &'static str = "query";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("box"))
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        dot::LabelText::label(format!("query: {}", self.name))
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        dot::Style::Bold
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("gold"))
    }
}

impl StyleableNode for plan::Context {
    const ID_PREFIX: &'static str = "context";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("invhouse"))
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        dot::LabelText::label("context")
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        dot::Style::Bold
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("gold"))
    }
}

#[enumtrait::store(operator_description_trait)]
pub trait OperatorDescription {
    fn description(&self, plan: &plan::Plan) -> String;
}

#[enumtrait::impl_trait(operator_description_trait for plan::operator_enum)]
impl OperatorDescription for plan::Operator {}

impl OperatorDescription for plan::Update {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Update")
    }
}

impl OperatorDescription for plan::Insert {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Insert")
    }
}

impl OperatorDescription for plan::Delete {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Delete")
    }
}

impl OperatorDescription for plan::GetUnique {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("GetUnique")
    }
}

impl OperatorDescription for plan::ScanRefs {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Scan")
    }
}

impl OperatorDescription for plan::DeRef {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("DeRef")
    }
}

impl OperatorDescription for plan::Map {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Map")
    }
}

impl OperatorDescription for plan::Expand {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Map")
    }
}

impl OperatorDescription for plan::Fold {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Fold")
    }
}

impl OperatorDescription for plan::Filter {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Filter")
    }
}

impl OperatorDescription for plan::Sort {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Sort")
    }
}

impl OperatorDescription for plan::Assert {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Assert")
    }
}

impl OperatorDescription for plan::Collect {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Collect")
    }
}

impl OperatorDescription for plan::Take {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Take")
    }
}

impl OperatorDescription for plan::Join {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Join")
    }
}

impl OperatorDescription for plan::GroupBy {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("GroupBy")
    }
}

impl OperatorDescription for plan::Fork {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Fork")
    }
}

impl OperatorDescription for plan::Union {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Union")
    }
}

impl OperatorDescription for plan::Row {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Row")
    }
}

impl OperatorDescription for plan::Return {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Return")
    }
}

impl OperatorDescription for plan::Discard {
    fn description(&self,plan: &plan::Plan) -> String {
        format!("Discard")
    }
}
