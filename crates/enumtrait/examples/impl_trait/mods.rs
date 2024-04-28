
mod ops {
    pub struct Map;
    pub struct Fold;
    pub struct Filter;
    
    #[enumtrait::quick_enum]
    #[enumtrait::store(pub operator_enum)]
    pub enum Operator {
        Map,
        Fold,
        Filter
    }
}

mod codegen {
    #[enumtrait::store(codegen_trait)]
    trait CodeGen {
        fn generate(self);
    }
    
    impl CodeGen for super::ops::Map { fn generate(self){} }
    impl CodeGen for super::ops::Fold { fn generate(self){} }
    impl CodeGen for super::ops::Filter { fn generate(self){} }
    
    #[enumtrait::impl_trait(codegen_trait for super::ops::operator_enum)]
    impl CodeGen for super::ops::Operator {}
}


fn main() {}