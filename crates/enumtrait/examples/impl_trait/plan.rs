struct Map;
struct Fold;
struct Filter;

#[enumtrait::quick_enum]
#[enumtrait::store(operator_enum)]
enum Operator {
    Map,
    Fold,
    Filter
}

#[enumtrait::store(codegen_trait)]
trait CodeGen {
    fn generate(self) -> String;
}

impl CodeGen for Map {
    fn generate(self) -> String {
        "map".to_string()
    }
}

impl CodeGen for Fold {
    fn generate(self) -> String {
        "fold".to_string()
    }
}

impl CodeGen for Filter {
    fn generate(self) -> String {
        "filter".to_string()
    }
}

#[enumtrait::impl_trait(codegen_trait for operator_enum)]
impl CodeGen for Operator {}

fn main() {}