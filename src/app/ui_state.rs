#[derive(PartialEq)]
pub enum ActivePanel {
    Tree,
    Details,
}

#[derive(PartialEq, Clone)]
pub enum DetailField {
    Url,
    Body,
    Params,
    Headers,
    AuthType,
    AuthUsername,
    AuthPassword,
    None,
}

#[derive(PartialEq)]
pub enum HeaderInputMode {
    Key,
    Value,
}

#[derive(PartialEq)]
pub enum ParameterInputMode {
    Key,
    Value,
}
