#![allow(clippy::useless_format)] // TODO: so clippy will pass while the nodes are not labelled
use crate::plan::{self, With};
use dot;

#[derive(Clone)]
pub enum PlanNode {
    Table(plan::Key<plan::Table>),
    Operator(plan::Key<plan::Operator>),
    Dataflow(plan::Key<plan::DataFlow>),
    Query(plan::Key<plan::Query>),
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
        let description = match &self.kind {
            plan::OperatorKind::Modify { modify_after, op } => op.description(plan),
            plan::OperatorKind::Access { access_after, op } => op.description(plan),
            plan::OperatorKind::Pure(op) => op.description(plan),
            plan::OperatorKind::Flow(op) => op.description(plan),
        };

        dot::LabelText::label(description)
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
        Some(dot::LabelText::label("box"))
    }

    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a> {
        if let plan::DataFlow::Conn {
            from,
            to,
            with
        } = self {
            dot::LabelText::label(format!("{}", With { plan, extended: with }))
        } else {
            unreachable!("Only `DataFlow::Conn` edges should be present in dataflow")
        }
    }

    fn style(&self, plan: &plan::Plan) -> dot::Style {
        if let plan::DataFlow::Conn { from, to, with } = self {
            if with.stream {
                dot::Style::Bold
            } else {
                dot::Style::None
            }
        } else {
            unreachable!()
        }
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("darkorchid1"))
    }
}

impl StyleableNode for plan::Query {
    const ID_PREFIX: &'static str = "query";

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("invhouse"))
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

#[enumtrait::store(operator_description_trait)]
pub trait OperatorDescription {
    fn description(&self, plan: &plan::Plan) -> String;
}

#[enumtrait::impl_trait(operator_description_trait for plan::modify_operator_enum)]
impl OperatorDescription for plan::ModifyOperator {}

#[enumtrait::impl_trait(operator_description_trait for plan::access_operator_enum)]
impl OperatorDescription for plan::AccessOperator {}

#[enumtrait::impl_trait(operator_description_trait for plan::pure_operator_enum)]
impl OperatorDescription for plan::PureOperator {}

#[enumtrait::impl_trait(operator_description_trait for plan::flow_operator_enum)]
impl OperatorDescription for plan::FlowOperator {}

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

impl OperatorDescription for plan::Scan {
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
