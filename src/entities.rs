#[derive(Debug)]
pub enum VariableType {
    Integer(i32),
    Float(f32),
    Character(char),
}

#[derive(Debug)]
pub enum Token {
    Und,
    Pred,
    Lit(VariableType),
    Var(VariableType)
}
//Floating Point ===================================================================================
#[derive(Debug)]
pub enum PointState {
    None,
    Error(String),
    Token(i16),
}

#[derive(Debug)]
pub struct MetaData{
    pub line: u32,
    pub raw: String
}

#[derive(Debug)]
pub struct FPoint{
    pub meta_data: MetaData,
    pub state: PointState,
}