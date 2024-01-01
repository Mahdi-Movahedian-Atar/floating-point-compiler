pub mod scanner{
    use my_macro::unique_i16;

    use crate::entities::{FPoint , MetaData, Token, PointState, VariableType};
    use crate::utility::utility::{s_hash};

    macro_rules! token_insert {
    ( $y:literal, $tokens: ident, $token: ident, $line_num: ident) => {
        {
            $tokens.0.push(FPoint{meta_data:MetaData
                {line:$line_num.clone(),raw:$token},
                    state:PointState::Token(unique_i16!($y))});
            $token = String::new();
        }
    };

    ( Error, $y:literal, $tokens: ident, $token: ident, $line_num: ident) => {
        {
            $tokens.0.push(FPoint{meta_data:MetaData
                {line:$line_num.clone(),raw:$token},
                    state:PointState::Error($y.to_string())});
            $token = String::new();
        }
    };

    ( $y:ident, $tokens: ident, $token: ident, $line_num: ident) => {
        {
            $tokens.0.push(FPoint{meta_data:MetaData
                {line:$line_num.clone(),raw:$token.clone()},
                    state:PointState::Token($y)});
        }
    };
}

    pub fn tokenizer(line:String, line_num: u32) -> (Vec<FPoint>, Vec<(i16, Token)>, i8) {
        let mut tokens:(Vec<FPoint>,Vec<(i16,Token)>,i8) = (Vec::new(),Vec::new(),0);

        let mut token: String = String::new();

        let length= line.len();
        let chars: Vec<char> = line.chars().collect();

        let mut i:usize = 0;
        while i < length {
            if tokens.2 > 0 { i += 1; continue;}
            token.push(chars[i]);
            match chars[i] {
                '\'' => {
                    i += 1;
                    if i + 1 < length {
                        token.push(chars[i]);
                        i += 1;
                        token.push(chars[i]);
                        i += 1;
                        if token.as_bytes()[2] as char == '\'' {
                            let token_hash = s_hash(token.as_str());

                            token_insert!(token_hash,tokens,token,line_num);
                            tokens.1.push(
                                (token_hash,Token::Lit(VariableType::Character(token.as_bytes()[1] as char))));
                            token = String::new();
                            continue;
                        }
                    }

                    token_insert!(Error, "Wrong literal character type",tokens,token,line_num);
                },
                '0'..='9' => {
                    i += 1;
                    let mut is_float = false;
                    while i < length {
                        match chars[i] {
                            '0'..='9' =>{
                                token.push(chars[i]);
                            }
                            '.' =>{
                                if is_float {i+=1;continue;}
                                is_float = true;
                                token.push('.');
                            }
                            _ =>{
                                i -= 1;
                                break;
                            }
                        }
                        i += 1;
                    }
                    let token_hash = s_hash(token.as_str());
                    if is_float {
                        match token.parse::<f32>() {
                            Ok(val) => {
                                token_insert!(token_hash,tokens,token,line_num);
                                tokens.1.push(
                                    (token_hash,Token::Lit(VariableType::Float(val))));
                            },
                            Err(_) => {
                                token_insert!(Error,"Could no parse the float type",tokens,token,line_num);
                            }
                        };
                    }
                    else {
                        match token.parse::<i32>() {
                            Ok(val) => {
                                token_insert!(token_hash,tokens,token,line_num);
                                tokens.1.push(
                                    (token_hash,Token::Lit(VariableType::Integer(val))));
                            },
                            Err(_) => {
                                token_insert!(Error,"Could no parse the integer type",tokens,token,line_num);
                            }
                        };
                    }
                    token = String::new();
                },
                ';' => token_insert!(";",tokens,token,line_num),
                ':' => token_insert!(":",tokens,token,line_num),
                '(' => token_insert!("(",tokens,token,line_num),
                ')' => token_insert!(")",tokens,token,line_num),
                '{' => token_insert!("{",tokens,token,line_num),
                '}' => token_insert!("}",tokens,token,line_num),
                '[' => token_insert!("[",tokens,token,line_num),
                ']' => token_insert!("]",tokens,token,line_num),
                ',' => token_insert!(",",tokens,token,line_num),
                '+' => token_insert!("+",tokens,token,line_num),
                '-' => token_insert!("-",tokens,token,line_num),
                '*' => {
                    if i+1 < length { if chars[i + 1] == '/'{
                        token = String::new();
                        i += 1;
                        tokens.2 += -1;
                        continue;
                    } }
                    token_insert!("*",tokens,token,line_num);
                },
                '/' => {
                    if i+1< length { if chars[i + 1] == '*'{
                        token = String::new();
                        i += 1;
                        tokens.2 += 1;
                        continue;
                    } }
                    token_insert!("/",tokens,token,line_num);
                },
                '=' => token_insert!("=",tokens,token,line_num),
                '|' => token_insert!("=",tokens,token,line_num),
                ' ' => {token = String::new();}
                _ => {
                    i += 1;
                    while i < length {
                        match chars[i] {
                            ';' |
                            ':' |
                            '(' |
                            ')' |
                            '{' |
                            '}' |
                            '[' |
                            ']' |
                            ',' |
                            '+' |
                            '-' |
                            '*' |
                            '/' |
                            '=' |
                            '|' |
                            ' ' |
                            '\'' => { i -= 1; break; },
                            _ => { token.push(chars[i]);i += 1; }
                        }
                    }
                    let token_hash = s_hash(token.as_str());
                    token_insert!(token_hash,tokens,token,line_num);
                    tokens.1.push((token_hash,Token::Und));
                    token = String::new();
                }
            }
            if tokens.2 < 0 { tokens.0 = Vec::new(); tokens.1 = Vec::new(); }
            i += 1;
        }
        return tokens;
    }
}