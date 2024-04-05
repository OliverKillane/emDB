// struct Map;
// struct Fold;
// struct Filter;

// #[enumtrait::get_enum(operator_enum)]
// enum Operator {
//     Map,
//     Fold,
//     Filter
// }

// #[enumtrait::get_trait(operator_enum => operator_codegen)]
// trait CodeGen {
//     fn generate(self) -> String;
// }

// impl CodeGen for Map {
//     fn generate(self) -> String {
//         "map".to_string()
//     }
// }

// impl CodeGen for Fold {
//     fn generate(self) -> String {
//         "fold".to_string()
//     }
// }

// impl CodeGen for Filter {
//     fn generate(self) -> String {
//         "filter".to_string()
//     }
// }

// #[enumtrait::impl_trait(operator_codegen)]
// impl CodeGen for Operator {}
