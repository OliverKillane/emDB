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
    fn id<'a>(&'a self, self_key: plan::Key<Self>) -> dot::Id<'a> {
        dot::Id::new(format!("{}_{}", Self::ID_PREFIX, self_key.arr_idx() )).unwrap()
    }

    fn shape<'a>(&'a self, plan: &plan::Plan) -> Option<dot::LabelText<'a>>;
    fn label<'a>(&'a self, plan: &plan::Plan) -> dot::LabelText<'a>;
    fn style(&self, plan: &plan::Plan) -> dot::Style;
    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>>;
}

pub trait OperatorDescription {
    fn description(&self, plan: &plan::Plan) -> String;
}

impl OperatorDescription for plan::PureOperator {
    fn description(&self, plan: &plan::Plan) -> String {
        String::from("pure")
    }
}

impl OperatorDescription for plan::FlowOperator {
    fn description(&self, plan: &plan::Plan) -> String {
        String::from("flow")
    }
}

impl OperatorDescription for plan::AccessOperator {
    fn description(&self, plan: &plan::Plan) -> String {
        String::from("access")
    }
}

impl OperatorDescription for plan::ModifyOperator {
    fn description(&self, plan: &plan::Plan) -> String {
        String::from("modify")
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
        Some(dot::LabelText::label("ellipse"))
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
        Some(dot::LabelText::label("beige"))
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
        dot::Style::Bold
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
        dot::Style::Dotted
    }

    fn color<'a>(&self, plan: &plan::Plan) -> Option<dot::LabelText<'a>> {
        Some(dot::LabelText::label("azure"))
    }
}