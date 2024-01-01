pub mod parser {
    use std::collections::{BTreeMap};
    use std::ptr::write;
    use my_macro::unique_i16;
    use crate::entities::{FPoint, MetaData, PointState, Precedence, Token, VariableType};

    impl FPoint{
        fn to_parsed(&mut self) -> FPoint{
            match self.state {
                PointState::Token(num) => {
                    self.state = PointState::ParsedToken(num);
                }
                _ => {}
            }
            self.clone()
        }
        fn to_error(&mut self, err: String) -> FPoint {
            self.state = PointState::Error(err);
            self.clone()
        }
        fn merge(points: Vec<FPoint>, point_state: PointState) -> FPoint{
            let mut raw = String::new();
            for point in &points {
                raw.push_str(&format!("{}", point.meta_data.raw));
                raw.push_str(" ");
            }

            return FPoint{meta_data:MetaData{line: points[0].meta_data.line,raw }, state:point_state};
        }
    }
    //===================================================================================
    #[derive(PartialEq, Debug)]
    enum TreeType{
        Token(i16,FPoint),
        Skip(i16)
    }
    #[derive(Debug)]
    struct ParserTree{
        token: TreeType,
        left_tokens: Option<Box<ParserTree>>,
        right_tokens: Option<Box<ParserTree>>,
    }

    impl ParserTree{
        fn new(token: TreeType) -> Self {
            ParserTree{token, left_tokens: None,right_tokens: None}
        }
        fn iter_recursive<T>(&self, collector: &mut T) where T: FnMut(FPoint),
        {
            if let Some(left) = &self.left_tokens {
                left.iter_recursive(collector);
            }

            match &self.token {
                TreeType::Token(.., value) => {
                    collector(value.clone());
                }
                TreeType::Skip(_) => {}
            }

            if let Some(right) = &self.right_tokens {
                right.iter_recursive(collector);
            }
        }
        fn iter<T>(&self, collector: &mut T) where T: FnMut(FPoint),
        {
            self.iter_recursive(collector);
        }
        fn add_left(&mut self, left_tree: ParserTree) {
            match &mut self.left_tokens {
                (Some(s)) => s.add_left(left_tree),
                _ => self.left_tokens = Some(Box::new(left_tree))
            }
        }
        fn add_right(&mut self, right_tree: ParserTree) {
            match &mut self.right_tokens {
                (Some(s)) => s.add_left(right_tree),
                _ => self.right_tokens = Some(Box::new(right_tree))
            }
        }
    }

    macro_rules! pH {
        (error $line:expr, $error:literal, $($raw:expr),+) => {
            {let mut result = String::new();
            $(result.push_str(&format!("{}", $raw)); result.push_str(" ");)*

            FPoint{meta_data:MetaData{
                line:$line,raw:(result)},
                state:PointState::Error($error.to_string())}}
        };

        /*(Merge $point:ident $($pointE:ident)*,) => {
            FPoint{meta_data:MetaData{line: $point.meta_data.line}}
        };*/

        (hMatch $id:ident $hash:literal) => {
            *$id == PointState::Token(unique_i16!($hash))
        };
        (hMatch $id1:ident $id2:ident $hash1:literal $hash2:literal) => {
            *$id1 == PointState::Token(unique_i16!($hash1)) && $id2 == Some(&unique_i16!($hash2))
        };
        (sMatch $id:ident $hash:literal) => {
            $id == Some(&unique_i16!($hash))
        };
    }

    pub fn parser(mut points: Vec<FPoint>,mut token_tree: &mut BTreeMap<i16, Token>,precedence_tree: &BTreeMap<i16,BTreeMap<i16,Precedence>>) -> Vec<FPoint> {
        let mut new: Vec<FPoint> = Vec::new();
        let mut stack: Vec<i16> = Vec::new();

        {
            if points.len() < 3 {
                new.push(pH!(error 1, "no program found", "no program found"));
                return new;
            }
            if points[0].state != PointState::Token(unique_i16!("program")) ||
                points[2].state != PointState::Token(unique_i16!(";")) {
                new.push(pH!(error 1, "bad input", &points[0].meta_data.raw, &points[1].meta_data.raw, &points[1].meta_data.raw));
                return new;
            }

            points[0].to_parsed();
            match points[1].state {
                PointState::Token(num) => {
                    token_tree.get_mut(&num).map(|val| { *val = Token::Pred; });
                    points[1].state = PointState::ParsedToken(num);
                }
                _ => {}
            }
            points[2].to_parsed();

            new.push(FPoint::merge(points.drain(0..3).collect(), PointState::ParsedToken(unique_i16!("program"))));
        }

        let mut i : usize = 0;

        while i < points.len() {
            match (&points[i].state,stack.last()) {
                (s,None) if pH!(hMatch s "data") =>{
                    if points.len() - i < 3 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    }else if (points[i+1].state != PointState::Token(unique_i16!("division"))||
                        points[i+2].state != PointState::Token(unique_i16!(";"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: data division;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw,
                            &points[i+2].meta_data.raw))
                    }else {
                        new.push(FPoint::merge(points.drain(i..i+3).collect(), PointState::ParsedToken(unique_i16!("data"))));
                        stack.push(unique_i16!("data"));
                        continue;}
                    i+=1;
                }
                (s,k) if pH!(hMatch s k "end" "data") =>{
                    if points.len() - i < 1 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    }else if (points[i+1].state != PointState::Token(unique_i16!(";"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: end;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw))
                    } else {
                        new.push(FPoint::merge(points.drain(i..i+2).collect(), PointState::ParsedToken(unique_i16!("end"))));
                        stack.pop();
                        continue;}
                    i+=1;
                }
                (s,None) if pH!(hMatch s "procedure") =>{
                    if points.len() - i < 2 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    }else if (points[i+1].state != PointState::Token(unique_i16!("division"))||
                        points[i+2].state != PointState::Token(unique_i16!(";"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: procedure division;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw,
                            &points[i+2].meta_data.raw))
                    }else{
                        new.push(FPoint::merge(points.drain(i..i+3).collect(), PointState::ParsedToken(unique_i16!("procedure"))));
                        stack.push(unique_i16!("procedure"));
                        continue;}
                    i+=1;
                }
                (s,k) if pH!(hMatch s k "end" "procedure") =>{
                    if points.len() - i < 1 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    }else if (points[i+1].state != PointState::Token(unique_i16!(";"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: end;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw))
                    } else {
                        new.push(FPoint::merge(points.drain(i..i+2).collect(), PointState::ParsedToken(unique_i16!("end"))));
                        stack.pop();
                        continue;}
                    i+=1;
                }
                (_,k) if  pH!(sMatch k "data") =>{
                    let mut point_ref:Vec<&FPoint> = Vec::new();
                    let j = i.clone();
                    let mut state=',';
                    let mut error = String::new();

                    while i < points.len() {
                        match (&points[i].state,&state) {
                            (_,',') => {point_ref.push(&points[i]); i+= 1;state = ':'; }
                            (s,':') if pH!(hMatch s ",") => {i+= 1;state = ','; }
                            (s,':') if pH!(hMatch s ":") => {i+= 1;state = 't'; }
                            (_,'t') => {point_ref.push(&points[i]);i+= 1;state = ';'; }
                            (s,';') if pH!(hMatch s ";") => {i+= 1;state = 'A'; break; }

                            (s,'E') if pH!(hMatch s ";") => {i+= 1; break; }
                            (_,'E') => {i+= 1; }
                            _ => {i+= 1;state = 'E'; error="expected: <var> ,... : <type>;".to_string() }
                        }
                    };
                    let mut point_token:Vec<i16> = Vec::new();

                    if state == 'A' {
                        for point in &point_ref {
                            match (point.state) {
                                (PointState::Token(s)) => {
                                    point_token.push(s);
                                }
                                _ =>{ state = 'E'; error="bad input".to_string(); break; }
                            }
                        }
                    }

                    let mut var_type:i8 = 0;
                    if state == 'A' {
                        match point_token.last() {
                            (Some(s)) if *s == unique_i16!("integer") => { var_type = 0; point_token.pop();},
                            (Some(s)) if *s == unique_i16!("float") => { var_type = 1; point_token.pop();},
                            (Some(s)) if *s == unique_i16!("char") => { var_type = 2; point_token.pop();},
                            _ => {state = 'E'; error="type must be: integer or float or char".to_string();}
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            match token_tree[point] {
                                Token::Pred | Token::Lit(..) =>{ state = 'E'; error="cannot use predefined or literal expressions".to_string(); break; },
                                Token::Var(..) =>{state = 'E'; error="cannot redefine variables".to_string(); break;}
                                _ =>{}
                            }
                            match var_type {
                                0 => {
                                    token_tree.entry(*point).and_modify(|existing_token| {
                                        *existing_token = Token::Var(VariableType::Integer(0));
                                    });
                                }
                                1 => {
                                    token_tree.entry(*point).and_modify(|existing_token| {
                                        *existing_token = Token::Var(VariableType::Float(0.));
                                    });
                                }
                                2 => {
                                    token_tree.entry(*point).and_modify(|existing_token| {
                                        *existing_token = Token::Var(VariableType::Character('0'));
                                    });
                                }
                                _ => {}
                            }
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            new.push(FPoint{
                                meta_data:MetaData{
                                    line:point_ref[0].meta_data.line,
                                    raw: "val".to_string()
                                },
                                state: PointState::ParsedToken(point.clone())
                            })
                        }
                        continue;
                    }

                    if state == 'E' {
                        new.push(FPoint::merge(points.drain(j..i).collect(), PointState::ParsedToken(unique_i16!("data"))).to_error(error));
                        i = j;
                    }
                }
                (s,k) if pH!(hMatch s k "get" "procedure") =>{
                    let mut point_ref:Vec<&FPoint> = Vec::new();
                    let j = i.clone();
                    let mut state=',';
                    let mut error = String::new();

                    while i < points.len() {
                        i+= 1;
                        match (&points[i].state,&state) {
                            (_,',') => {point_ref.push(&points[i]);state = ':'; }
                            (s,':') if pH!(hMatch s ",") => {state = ','; }
                            (s,':') if pH!(hMatch s ";") => {state = 'A'; break; }

                            (s,'E') if pH!(hMatch s ";") => {break; }
                            (_,'E') => {}
                            _ => {state = 'E'; error="expected: get <var> ,... ;".to_string() }
                        }
                    };
                    let mut point_token:Vec<i16> = Vec::new();

                    if state == 'A' {
                        for point in &point_ref {
                            match (point.state) {
                                (PointState::Token(s)) => {
                                    point_token.push(s);
                                }
                                _ =>{ state = 'E'; error="bad input".to_string(); break; }
                            }
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            println!("{:?}",token_tree[point]);
                            match token_tree[point] {
                                Token::Lit(..) | Token::Pred =>{state = 'E'; error="cannot use predefined or literal expressions".to_string(); break;},
                                Token::Und => {state = 'E'; error="cannot use undefined variables".to_string(); break;}
                                _ =>{}
                            }
                        }
                    }

                    if state == 'A' {
                        let root_point = points[j].clone().to_parsed();
                        new.push(root_point);
                        for point in &point_token {
                            new.push(FPoint{
                                meta_data:MetaData{
                                    line:point_ref[0].meta_data.line,
                                    raw: "val".to_string()
                                },
                                state: PointState::ParsedToken(point.clone())
                            })
                        }
                        continue;
                    }

                    if state == 'E' {
                        new.push(FPoint::merge(points.drain(j..i).collect(), PointState::ParsedToken(unique_i16!("get"))).to_error(error));
                        i = j;
                    }
                }
                (s,k) if pH!(hMatch s k "put" "procedure") =>{
                    let mut point_ref:Vec<&FPoint> = Vec::new();
                    let j = i.clone();
                    let mut state=',';
                    let mut error = String::new();

                    while i < points.len() {
                        i+= 1;
                        match (&points[i].state,&state) {
                            (_,',') => {point_ref.push(&points[i]);state = ':'; }
                            (s,':') if pH!(hMatch s ",") => {state = ','; }
                            (s,':') if pH!(hMatch s ";") => {state = 'A'; break; }

                            (s,'E') if pH!(hMatch s ";") => {break; }
                            (_,'E') => {}
                            _ => {state = 'E'; error="expected: get <var> ,... ;".to_string() }
                        }
                    };
                    let mut point_token:Vec<i16> = Vec::new();

                    if state == 'A' {
                        for point in &point_ref {
                            match (point.state) {
                                (PointState::Token(s)) => {
                                    point_token.push(s);
                                }
                                _ =>{ state = 'E'; error="bad input".to_string(); break; }
                            }
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            println!("{:?}",token_tree[point]);
                            match token_tree[point] {
                                Token::Lit(..) | Token::Pred =>{state = 'E'; error="cannot use predefined or literal expressions".to_string(); break;},
                                Token::Und => {state = 'E'; error="cannot use undefined variables".to_string(); break;}
                                _ =>{}
                            }
                        }
                    }

                    if state == 'A' {
                        let root_point = points[j].clone().to_parsed();
                        new.push(root_point);
                        for point in &point_token {
                            new.push(FPoint{
                                meta_data:MetaData{
                                    line:point_ref[0].meta_data.line,
                                    raw: "val".to_string()
                                },
                                state: PointState::ParsedToken(point.clone())
                            })
                        }
                        continue;
                    }

                    if state == 'E' {
                        new.push(FPoint::merge(points.drain(j..i).collect(), PointState::ParsedToken(unique_i16!("get"))).to_error(error));
                        i = j;
                    }
                }
                (s,k) if pH!(hMatch s k "set" "procedure") =>{
                    if points.len() - i < 5 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    }else if (
                        match points[i+1].state {
                        (PointState::Token(s)) => match token_tree[&s] { (Token::Var(..)) => false, _=> true },
                            _ => true }
                            || points[i+2].state != PointState::Token(unique_i16!("to"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: set <var> to",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw,
                            &points[i+2].meta_data.raw))
                    }else{
                        new.push(points[i+1].clone().to_parsed());
                        match points[i+1].state {
                            (PointState::Token(s)) => match &token_tree[&s] { (Token::Var(s)) =>
                                match s {
                                    VariableType::Integer(..) => stack.push(unique_i16!("Integer")),
                                    VariableType::Character(..) => stack.push(unique_i16!("Character")),
                                    VariableType::Float(..) => stack.push(unique_i16!("Float"))
                                } _ =>{} },
                            _ =>{}
                        }
                        new.push(FPoint::merge(points.drain(i..i+3).collect(), PointState::ParsedToken(unique_i16!("="))));
                        stack.push(unique_i16!("math"));
                        continue;}
                    i+=1;
                }
                (_,k) if pH!(sMatch k "math") =>{
                    let j = i.clone();
                    stack.pop();

                    let math_type = stack.pop();
                    let mut tree_stack:Vec<ParserTree> = Vec::from([ParserTree::new(TreeType::Skip(unique_i16!("S")))]);
                    let mut tree_len:usize = 0;
                    let mut has_ended = false;
                    let mut error_massage = String::new();

                    while i <  points.len(){
                        if has_ended { break; }
                        while tree_stack.len() > 1 {
                            let f_token = match &tree_stack[tree_len-1].token {
                                (TreeType::Token(s,_)) => s,
                                (TreeType::Skip(s)) => s
                            };
                            let s_token =match &tree_stack[tree_len].token {
                                (TreeType::Token(s,_)) => s,
                                (TreeType::Skip(s)) => s
                            };

                            match &precedence_tree.get(f_token) {
                                (Some(t)) => match t.get(s_token) {
                                    (Some(Precedence::Lesser)) => {break; },
                                    (Some(Precedence::Greater)) => {
                                        let new = tree_stack.remove(tree_len-1);
                                        tree_stack[tree_len - 1].add_right(new);
                                        tree_len -=1;
                                    }
                                    (Some(Precedence::Needs(s))) => {
                                        let mut new = ParserTree::new(TreeType::Skip(s.clone()));
                                        new.add_right(tree_stack.pop().unwrap());
                                        new.add_left(tree_stack.pop().unwrap());
                                        tree_stack.push(new);
                                        tree_len -=1;
                                        if tree_len == 0 { has_ended = true; break; }
                                    }
                                    _=>{has_ended = true; break;}
                                }
                                _=>{has_ended = true; break;}
                            }
                        }

                        if has_ended { break; }
                        tree_len +=1;

                        match points[i].state {
                            PointState::Token(s) => match (&token_tree[&s],math_type) {
                                (Token::Pred,_) => {
                                    if s == unique_i16!("+") || s == unique_i16!("-") || s == unique_i16!("*") ||
                                        s == unique_i16!("/") || s == unique_i16!("(") || s == unique_i16!(")")
                                        {
                                            tree_stack.push(ParserTree::new(TreeType::Token(s, points[i].to_parsed())))
                                        } else { tree_stack.push(ParserTree::new(TreeType::Skip(unique_i16!(";")))) }
                                },
                                (Token::Var(VariableType::Integer(_)),Some(unique_i16!("Integer"))) =>
                                    tree_stack.push(ParserTree::new(TreeType::Token(unique_i16!("id"),points[i].to_parsed()))),
                                (Token::Var(VariableType::Float(_)),Some(unique_i16!("Float"))) =>
                                    tree_stack.push(ParserTree::new(TreeType::Token(unique_i16!("id"),points[i].to_parsed()))),
                                (Token::Var(VariableType::Character(_)),Some(unique_i16!("Character"))) =>
                                    tree_stack.push(ParserTree::new(TreeType::Token(unique_i16!("id"),points[i].to_parsed()))),
                                (Token::Lit(VariableType::Integer(_)),Some(unique_i16!("Integer"))) =>
                                    tree_stack.push(ParserTree::new(TreeType::Token(unique_i16!("id"),points[i].to_parsed()))),
                                (Token::Lit(VariableType::Float(_)),Some(unique_i16!("Float"))) =>
                                    tree_stack.push(ParserTree::new(TreeType::Token(unique_i16!("id"),points[i].to_parsed()))),
                                (Token::Lit(VariableType::Character(_)),Some(unique_i16!("Character"))) =>
                                    tree_stack.push(ParserTree::new(TreeType::Token(unique_i16!("id"),points[i].to_parsed()))),
                                (_) => { has_ended = true; error_massage = "wrong type of variable or literal".to_string(); }
                            },
                            _ => {has_ended = true; error_massage = "bad input".to_string();}
                        }
                        i += 1;
                    }
                    if tree_stack.len() != 1 || tree_stack[0].token != TreeType::Skip(unique_i16!("P")) {
                        if points[i-1].state != PointState::Token(unique_i16!(";")) {
                            while i <  points.len(){
                                if points[i-1].state != PointState::Token(unique_i16!(";")){break;}
                                i+=1;
                            }
                        }
                        new.push(FPoint::merge(points.drain(j..i).collect(), PointState::ParsedToken(unique_i16!("math"))).to_error(error_massage));
                        i = j;
                    }
                    else {
                        let mut collector = |value: FPoint| {
                            println!("{:?}",&value);
                            new.push(value);
                        };

                        // Call the iter function on your tree, providing the collector closure
                        tree_stack[0].iter(&mut collector);
                    }

                }
                _ => {
                    i +=1;
                }
            }
        }

        return new;
    }
}