pub mod parser {
    use crate::entities::{FPoint, MetaData, PointState, Precedence, Token, VariableType};
    use my_macro::unique_i16;
    use std::collections::BTreeMap;

    impl FPoint {
        fn to_parsed(&mut self) -> FPoint {
            let mut parsed = self.clone();
            match parsed.state {
                PointState::Token(num) => {
                    parsed.state = PointState::ParsedToken(num);
                }
                _ => {}
            }
            parsed
        }
        fn to_error(&mut self, err: String) -> FPoint {
            self.state = PointState::Error(err);
            self.clone()
        }
        fn merge(points: Vec<FPoint>, point_state: PointState) -> FPoint {
            let mut raw = String::new();
            for point in &points {
                raw.push_str(&format!("{}", point.meta_data.raw));
                raw.push_str(" ");
            }

            return FPoint {
                meta_data: MetaData {
                    line: points[0].meta_data.line,
                    raw,
                },
                state: point_state,
            };
        }
    }
    //===================================================================================
    #[derive(PartialEq, Debug)]
    enum TreeType {
        Token(i16, FPoint),
        Skip(i16),
    }
    #[derive(Debug)]
    struct ParserTree {
        token: TreeType,
        left_tokens: Option<Box<ParserTree>>,
        right_tokens: Option<Box<ParserTree>>,
    }

    impl ParserTree {
        fn new(token: TreeType) -> Self {
            ParserTree {
                token,
                left_tokens: None,
                right_tokens: None,
            }
        }
        fn iter_recursive<T>(&self, collector: &mut T)
        where
            T: FnMut(FPoint),
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
        fn iter<T>(&self, collector: &mut T)
        where
            T: FnMut(FPoint),
        {
            self.iter_recursive(collector);
        }
        fn add_left(&mut self, left_tree: ParserTree) {
            match &mut self.left_tokens {
                (Some(s)) => s.add_left(left_tree),
                _ => self.left_tokens = Some(Box::new(left_tree)),
            }
        }
        fn add_right(&mut self, right_tree: ParserTree) {
            match &mut self.right_tokens {
                (Some(s)) => s.add_left(right_tree),
                _ => self.right_tokens = Some(Box::new(right_tree)),
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
        (hMatch $id1:ident $id2:ident $hash1:literal $hash2:literal $hash3:literal) => {
            *$id1 == PointState::Token(unique_i16!($hash1)) && ($id2 == Some(&unique_i16!($hash2)) || $id2 == Some(&unique_i16!($hash3)))
        };
        (sMatch $id:ident $hash:literal) => {
            $id == Some(&unique_i16!($hash))
        };
    }

    pub fn parser(
        mut points: Vec<FPoint>,
        mut token_tree: &mut BTreeMap<i16, Token>,
        precedence_tree: &BTreeMap<i16, BTreeMap<i16, Precedence>>,
    ) -> (Vec<FPoint>, bool) {
        let mut new: Vec<FPoint> = Vec::new();
        let mut stack: Vec<i16> = Vec::new();

        {
            if points.len() < 3 {
                new.push(pH!(error 1, "no program found", "no program found"));
                return (new,false);
            }
            if points[0].state != PointState::Token(unique_i16!("program"))
                || points[2].state != PointState::Token(unique_i16!(";"))
            {
                new.push(pH!(error 1, "bad input", &points[0].meta_data.raw, &points[1].meta_data.raw, &points[1].meta_data.raw));
                return (new,false);
            }

            points[0].to_parsed();
            match points[1].state {
                PointState::Token(num) => {
                    token_tree.get_mut(&num).map(|val| {
                        *val = Token::Pred;
                    });
                    points[1].state = PointState::ParsedToken(num);
                }
                _ => {}
            }
            points[2].to_parsed();

            new.push(FPoint::merge(
                points.drain(0..3).collect(),
                PointState::ParsedToken(unique_i16!("program")),
            ));
        }

        let mut i: usize = 0;

        while i < points.len() {
            match (&points[i].state, stack.last()) {
                (s, None) if pH!(hMatch s "data") => {
                    if points.len() - i < 3 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (points[i + 1].state != PointState::Token(unique_i16!("division"))
                        || points[i + 2].state != PointState::Token(unique_i16!(";")))
                    {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: data division;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw,
                            &points[i+2].meta_data.raw))
                    } else {
                        new.push(FPoint::merge(
                            points.drain(i..i + 3).collect(),
                            PointState::ParsedToken(unique_i16!("data")),
                        ));
                        stack.push(unique_i16!("data"));
                        continue;
                    }
                    i += 1;
                }
                (s, k) if pH!(hMatch s k "end" "data") => {
                    if points.len() - i < 1 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (points[i + 1].state != PointState::Token(unique_i16!(";"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: end;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw))
                    } else {
                        new.push(FPoint::merge(
                            points.drain(i..i + 2).collect(),
                            PointState::ParsedToken(unique_i16!("end")),
                        ));
                        stack.pop();
                        continue;
                    }
                    i += 1;
                }
                (s, None) if pH!(hMatch s "procedure") => {
                    if points.len() - i < 2 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (points[i + 1].state != PointState::Token(unique_i16!("division"))
                        || points[i + 2].state != PointState::Token(unique_i16!(";")))
                    {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: procedure division;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw,
                            &points[i+2].meta_data.raw))
                    } else {
                        new.push(FPoint::merge(
                            points.drain(i..i + 3).collect(),
                            PointState::ParsedToken(unique_i16!("procedure")),
                        ));
                        stack.push(unique_i16!("procedure"));
                        continue;
                    }
                    i += 1;
                }
                (s, k) if pH!(hMatch s k "end" "procedure") => {
                    if points.len() - i < 1 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (points[i + 1].state != PointState::Token(unique_i16!(";"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: end;",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw))
                    } else {
                        new.push(FPoint::merge(
                            points.drain(i..i + 2).collect(),
                            PointState::ParsedToken(unique_i16!("end")),
                        ));
                        stack.pop();
                        continue;
                    }
                    i += 1;
                }
                (_, k) if pH!(sMatch k "data") => {
                    let mut point_ref: Vec<&FPoint> = Vec::new();
                    let j = i.clone();
                    let mut state = ',';
                    let mut error = String::new();

                    while i < points.len() {
                        match (&points[i].state, &state) {
                            (_, ',') => {
                                point_ref.push(&points[i]);
                                i += 1;
                                state = ':';
                            }
                            (s, ':') if pH!(hMatch s ",") => {
                                i += 1;
                                state = ',';
                            }
                            (s, ':') if pH!(hMatch s ":") => {
                                i += 1;
                                state = 't';
                            }
                            (_, 't') => {
                                point_ref.push(&points[i]);
                                i += 1;
                                state = ';';
                            }
                            (s, ';') if pH!(hMatch s ";") => {
                                i += 1;
                                state = 'A';
                                break;
                            }

                            (s, 'E') if pH!(hMatch s ";") => {
                                i += 1;
                                break;
                            }
                            (_, 'E') => {
                                i += 1;
                            }
                            _ => {
                                i += 1;
                                state = 'E';
                                error = "expected: <var> ,... : <type>;".to_string()
                            }
                        }
                    }
                    let mut point_token: Vec<i16> = Vec::new();

                    if state == 'A' {
                        for point in &point_ref {
                            match (point.state) {
                                (PointState::Token(s)) => {
                                    point_token.push(s);
                                }
                                _ => {
                                    state = 'E';
                                    error = "bad input".to_string();
                                    break;
                                }
                            }
                        }
                    }

                    let mut var_type: i8 = 0;
                    if state == 'A' {
                        match point_token.last() {
                            (Some(s)) if *s == unique_i16!("integer") => {
                                var_type = 0;
                                point_token.pop();
                            }
                            (Some(s)) if *s == unique_i16!("float") => {
                                var_type = 1;
                                point_token.pop();
                            }
                            (Some(s)) if *s == unique_i16!("char") => {
                                var_type = 2;
                                point_token.pop();
                            }
                            _ => {
                                state = 'E';
                                error = "type must be: integer or float or char".to_string();
                            }
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            match token_tree[point] {
                                Token::Pred | Token::Lit(..) => {
                                    state = 'E';
                                    error =
                                        "cannot use predefined or literal expressions".to_string();
                                    break;
                                }
                                Token::Var(..) => {
                                    state = 'E';
                                    error = "cannot redefine variables".to_string();
                                    break;
                                }
                                _ => {}
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
                            new.push(FPoint {
                                meta_data: MetaData {
                                    line: point_ref[0].meta_data.line,
                                    raw: "val".to_string(),
                                },
                                state: PointState::ParsedToken(point.clone()),
                            })
                        }
                        continue;
                    }

                    if state == 'E' {
                        new.push(
                            FPoint::merge(
                                points.drain(j..i).collect(),
                                PointState::ParsedToken(unique_i16!("data")),
                            )
                            .to_error(error),
                        );
                        i = j;
                    }
                }
                (s, k) if pH!(hMatch s k "repeat" "procedure" "{") => {
                    if points.len() - i < 1 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (points[i + 1].state != PointState::Token(unique_i16!("{"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: repeat {",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw))
                    } else {
                        new.push(points[i].to_parsed());
                        new.push(points[i + 1].to_parsed());
                        i += 2;
                        stack.push(unique_i16!("repeat"));
                        continue;
                    }
                    i += 1;
                }
                (s, k) if pH!(hMatch s k "}" "repeat") => {
                    stack.pop();
                    let var_point: i16;
                    new.push(points[i].to_parsed());
                    if points.len() - i < 4 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else {
                        match &points[i + 2].state {
                            (s) if pH!(hMatch s "times") => {
                                match &points[i + 1].state {
                                    (PointState::Token(s)) => match token_tree[s] {
                                        (Token::Var(VariableType::Integer(_))) => {
                                            var_point = unique_i16!("Integer");
                                        }
                                        (Token::Var(VariableType::Float(_))) => {
                                            var_point = unique_i16!("Float");
                                        }
                                        (Token::Var(VariableType::Character(_))) => {
                                            var_point = unique_i16!("Character");
                                        }
                                        (Token::Lit(VariableType::Integer(_))) => {
                                            var_point = unique_i16!("Integer");
                                        }
                                        (Token::Lit(VariableType::Float(_))) => {
                                            var_point = unique_i16!("Float");
                                        }
                                        (Token::Lit(VariableType::Character(_))) => {
                                            var_point = unique_i16!("Character");
                                        }
                                        (_) => {
                                            new.push(pH!(
                                        error points[i + 1].meta_data.line,
                                        "bad input",
                                        &points[i + 1].meta_data.raw,
                                        &points[i + 2].meta_data.raw));
                                            i += 3;
                                            continue;
                                        }
                                    },
                                    _ => {
                                        new.push(pH!(
                                    error points[i + 1].meta_data.line,
                                    "bad input",
                                    &points[i + 1].meta_data.raw,
                                    &points[i + 2].meta_data.raw));
                                        i += 3;
                                        continue;
                                    }
                                }
                                if points[i + 3].state != PointState::Token(unique_i16!(";")) {
                                    new.push(pH!(
                                    error points[i + 3].meta_data.line,
                                    "expected: ;",
                                    &points[i + 3].meta_data.raw));
                                    i += 3;
                                    continue;
                                }
                                new.push(points[i + 2].to_parsed());
                                new.push(points[i + 1].to_parsed());
                            }
                            (PointState::Token(_)) => {
                                stack.push(unique_i16!("condition"));
                                stack.push(unique_i16!("condition"));
                                /*stack.push(unique_i16!("condition_end"));
                                stack.push(var_point);
                                stack.push(unique_i16!("math"));
                                stack.push(unique_i16!("operator"));
                                stack.push(var_point);
                                stack.push(unique_i16!("math"));*/
                                i += 1;
                                continue;
                            }
                            _ => {
                                new.push(pH!(
                                    error points[i + 1].meta_data.line,
                                    "bad input",
                                    &points[i + 1].meta_data.raw,
                                    &points[i + 2].meta_data.raw));
                            }
                        }
                        i += 3;
                    }
                }
                (s, k) if pH!(hMatch s k "execute" "procedure" "{") => {
                    if points.len() - i < 1 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (points[i + 1].state != PointState::Token(unique_i16!("{"))) {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: execute {",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw))
                    } else {
                        new.push(points[i].to_parsed());
                        new.push(points[i + 1].to_parsed());
                        i += 2;
                        stack.push(unique_i16!("execute"));
                        continue;
                    }
                    i += 1;
                }
                (s, k) if pH!(hMatch s k "}" "execute") => {
                    stack.pop();
                    let var_point: i16;
                    new.push(points[i].to_parsed());
                    if points.len() - i < 4 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else {
                        match &points[i + 2].state {
                            (PointState::Token(_)) => {
                                stack.push(unique_i16!("condition"));
                                stack.push(unique_i16!("condition"));
                                i += 1;
                                continue;
                            }
                            _ => {
                                new.push(pH!(
                                    error points[i + 1].meta_data.line,
                                    "bad input",
                                    &points[i + 1].meta_data.raw,
                                    &points[i + 2].meta_data.raw));
                            }
                        }
                        i += 3;
                    }
                }
                (s, k) if pH!(hMatch s k "get" "procedure" "repeat") => {
                    let mut point_ref: Vec<&FPoint> = Vec::new();
                    let j = i.clone();
                    let mut state = ',';
                    let mut error = String::new();

                    while i < points.len() {
                        i += 1;
                        match (&points[i].state, &state) {
                            (_, ',') => {
                                point_ref.push(&points[i]);
                                state = ':';
                            }
                            (s, ':') if pH!(hMatch s ",") => {
                                state = ',';
                            }
                            (s, ':') if pH!(hMatch s ";") => {
                                state = 'A';
                                break;
                            }

                            (s, 'E') if pH!(hMatch s ";") => {
                                break;
                            }
                            (_, 'E') => {}
                            _ => {
                                state = 'E';
                                error = "expected: get <var> ,... ;".to_string()
                            }
                        }
                    }
                    let mut point_token: Vec<i16> = Vec::new();

                    if state == 'A' {
                        for point in &point_ref {
                            match (point.state) {
                                (PointState::Token(s)) => {
                                    point_token.push(s);
                                }
                                _ => {
                                    state = 'E';
                                    error = "bad input".to_string();
                                    break;
                                }
                            }
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            match token_tree[point] {
                                Token::Lit(..) | Token::Pred => {
                                    state = 'E';
                                    error =
                                        "cannot use predefined or literal expressions".to_string();
                                    break;
                                }
                                Token::Und => {
                                    state = 'E';
                                    error = "cannot use undefined variables".to_string();
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }

                    if state == 'A' {
                        let root_point = points[j].clone().to_parsed();
                        new.push(root_point);
                        for point in &point_token {
                            new.push(FPoint {
                                meta_data: MetaData {
                                    line: point_ref[0].meta_data.line,
                                    raw: "val".to_string(),
                                },
                                state: PointState::ParsedToken(point.clone()),
                            })
                        }
                        continue;
                    }

                    if state == 'E' {
                        new.push(
                            FPoint::merge(
                                points.drain(j..i).collect(),
                                PointState::ParsedToken(unique_i16!("get")),
                            )
                            .to_error(error),
                        );
                        i = j;
                    }
                }
                (s, k) if pH!(hMatch s k "put" "procedure" "repeat") => {
                    let mut point_ref: Vec<&FPoint> = Vec::new();
                    let j = i.clone();
                    let mut state = ',';
                    let mut error = String::new();

                    while i < points.len() {
                        i += 1;
                        match (&points[i].state, &state) {
                            (_, ',') => {
                                point_ref.push(&points[i]);
                                state = ':';
                            }
                            (s, ':') if pH!(hMatch s ",") => {
                                state = ',';
                            }
                            (s, ':') if pH!(hMatch s ";") => {
                                state = 'A';
                                break;
                            }

                            (s, 'E') if pH!(hMatch s ";") => {
                                break;
                            }
                            (_, 'E') => {}
                            _ => {
                                state = 'E';
                                error = "expected: get <var> ,... ;".to_string()
                            }
                        }
                    }
                    let mut point_token: Vec<i16> = Vec::new();

                    if state == 'A' {
                        for point in &point_ref {
                            match (point.state) {
                                (PointState::Token(s)) => {
                                    point_token.push(s);
                                }
                                _ => {
                                    state = 'E';
                                    error = "bad input".to_string();
                                    break;
                                }
                            }
                        }
                    }

                    if state == 'A' {
                        for point in &point_token {
                            match token_tree[point] {
                                Token::Lit(..) | Token::Pred => {
                                    state = 'E';
                                    error =
                                        "cannot use predefined or literal expressions".to_string();
                                    break;
                                }
                                Token::Und => {
                                    state = 'E';
                                    error = "cannot use undefined variables".to_string();
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }

                    if state == 'A' {
                        let root_point = points[j].clone().to_parsed();
                        new.push(root_point);
                        for point in &point_token {
                            new.push(FPoint {
                                meta_data: MetaData {
                                    line: point_ref[0].meta_data.line,
                                    raw: "val".to_string(),
                                },
                                state: PointState::ParsedToken(point.clone()),
                            })
                        }
                        continue;
                    }

                    if state == 'E' {
                        new.push(
                            FPoint::merge(
                                points.drain(j..i).collect(),
                                PointState::ParsedToken(unique_i16!("get")),
                            )
                            .to_error(error),
                        );
                        i = j;
                    }
                }
                (s, k) if pH!(hMatch s k "set" "procedure" "repeat") => {
                    if points.len() - i < 5 {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "bad input", &points[i].meta_data.raw))
                    } else if (match points[i + 1].state {
                        (PointState::Token(s)) => match token_tree[&s] {
                            (Token::Var(..)) => false,
                            _ => true,
                        },
                        _ => true,
                    } || points[i + 2].state != PointState::Token(unique_i16!("to")))
                    {
                        new.push(pH!(
                            error points[i].meta_data.line,
                            "expected: set <var> to",
                            &points[i].meta_data.raw,
                            &points[i+1].meta_data.raw,
                            &points[i+2].meta_data.raw))
                    } else {
                        new.push(points[i + 1].clone().to_parsed());
                        stack.push(unique_i16!("set_end"));
                        match points[i + 1].state {
                            (PointState::Token(s)) => match &token_tree[&s] {
                                (Token::Var(s)) => match s {
                                    VariableType::Integer(..) => stack.push(unique_i16!("Integer")),
                                    VariableType::Character(..) => {
                                        stack.push(unique_i16!("Character"))
                                    }
                                    VariableType::Float(..) => stack.push(unique_i16!("Float")),
                                },
                                _ => {}
                            },
                            _ => {}
                        }
                        new.push(FPoint::merge(
                            points.drain(i..i + 3).collect(),
                            PointState::ParsedToken(unique_i16!("=")),
                        ));
                        stack.push(unique_i16!("math"));
                        continue;
                    }
                    i += 1;
                }
                (_, k) if pH!(sMatch k "math") => {
                    let j = i.clone();
                    stack.pop();

                    let math_type = stack.pop();
                    let mut tree_stack: Vec<ParserTree> =
                        Vec::from([ParserTree::new(TreeType::Skip(unique_i16!("S")))]);
                    let mut tree_len: usize = 0;
                    let mut has_ended = false;
                    let mut error_massage = String::new();

                    while i < points.len() {
                        if has_ended {
                            break;
                        }
                        while tree_stack.len() > 1 {
                            let f_token = match &tree_stack[tree_len - 1].token {
                                (TreeType::Token(s, _)) => s,
                                (TreeType::Skip(s)) => s,
                            };
                            let s_token = match &tree_stack[tree_len].token {
                                (TreeType::Token(s, _)) => s,
                                (TreeType::Skip(s)) => s,
                            };

                            match &precedence_tree.get(f_token) {
                                (Some(t)) => match t.get(s_token) {
                                    (Some(Precedence::Lesser)) => {
                                        break;
                                    }
                                    (Some(Precedence::Greater)) => {
                                        let new = tree_stack.remove(tree_len - 1);
                                        tree_stack[tree_len - 1].add_right(new);
                                        tree_len -= 1;
                                    }
                                    (Some(Precedence::Needs(s))) => {
                                        let mut new = ParserTree::new(TreeType::Skip(s.clone()));
                                        new.add_right(tree_stack.pop().unwrap());
                                        new.add_right(tree_stack.pop().unwrap());
                                        tree_stack.push(new);
                                        tree_len -= 1;
                                        if tree_len == 0 {
                                            has_ended = true;
                                            break;
                                        }
                                    }
                                    _ => {
                                        has_ended = true;
                                        break;
                                    }
                                },
                                _ => {
                                    has_ended = true;
                                    break;
                                }
                            }
                        }

                        if has_ended {
                            break;
                        }
                        tree_len += 1;

                        match points[i].state {
                            PointState::Token(s) => match (&token_tree[&s], math_type) {
                                (Token::Pred, _) => {
                                    if s == unique_i16!("+")
                                        || s == unique_i16!("-")
                                        || s == unique_i16!("*")
                                        || s == unique_i16!("/")
                                        || s == unique_i16!("(")
                                        || s == unique_i16!(")")
                                    {
                                        tree_stack.push(ParserTree::new(TreeType::Token(
                                            s,
                                            points[i].to_parsed(),
                                        )))
                                    } else {
                                        tree_stack
                                            .push(ParserTree::new(TreeType::Skip(unique_i16!(";"))))
                                    }
                                }
                                (
                                    Token::Var(VariableType::Integer(_)),
                                    Some(unique_i16!("Integer")),
                                ) => tree_stack.push(ParserTree::new(TreeType::Token(
                                    unique_i16!("id"),
                                    points[i].to_parsed(),
                                ))),
                                (
                                    Token::Var(VariableType::Float(_)),
                                    Some(unique_i16!("Float")),
                                ) => tree_stack.push(ParserTree::new(TreeType::Token(
                                    unique_i16!("id"),
                                    points[i].to_parsed(),
                                ))),
                                (
                                    Token::Var(VariableType::Character(_)),
                                    Some(unique_i16!("Character")),
                                ) => tree_stack.push(ParserTree::new(TreeType::Token(
                                    unique_i16!("id"),
                                    points[i].to_parsed(),
                                ))),
                                (
                                    Token::Lit(VariableType::Integer(_)),
                                    Some(unique_i16!("Integer")),
                                ) => tree_stack.push(ParserTree::new(TreeType::Token(
                                    unique_i16!("id"),
                                    points[i].to_parsed(),
                                ))),
                                (
                                    Token::Lit(VariableType::Float(_)),
                                    Some(unique_i16!("Float")),
                                ) => tree_stack.push(ParserTree::new(TreeType::Token(
                                    unique_i16!("id"),
                                    points[i].to_parsed(),
                                ))),
                                (
                                    Token::Lit(VariableType::Character(_)),
                                    Some(unique_i16!("Character")),
                                ) => tree_stack.push(ParserTree::new(TreeType::Token(
                                    unique_i16!("id"),
                                    points[i].to_parsed(),
                                ))),
                                (_) => {
                                    has_ended = true;
                                    error_massage = "wrong type of variable or literal".to_string();
                                }
                            },
                            _ => {
                                has_ended = true;
                                error_massage = "bad input".to_string();
                            }
                        }
                        i += 1;
                    }
                    if tree_stack.len() != 1
                        || tree_stack[0].token != TreeType::Skip(unique_i16!("P"))
                    {
                        if points[i - 1].state != PointState::Token(unique_i16!(";")) {
                            while i < points.len() {
                                if points[i - 1].state != PointState::Token(unique_i16!(";")) {
                                    break;
                                }
                                i += 1;
                            }
                            i -= 1;
                        }
                        if i >= points.len() {
                            i = points.len() - 1
                        }
                        if i == j {
                            i += 1
                        }
                        new.push(
                            FPoint::merge(
                                points.drain(j..i).collect(),
                                PointState::ParsedToken(unique_i16!("math")),
                            )
                            .to_error(error_massage),
                        );
                        i = j;
                    } else {
                        let mut collector = |value: FPoint| {
                            new.push(value);
                        };

                        tree_stack[0].iter(&mut collector);
                        i -= 1;
                    }
                }
                (_, k) if pH!(sMatch k "set_end") => {
                    stack.pop();
                    let mut last_point = points.remove(i);
                    match &last_point.state {
                        (PointState::Token(unique_i16!(";"))) => {
                            new.push(last_point.to_parsed());
                        }
                        _ => new.push(pH!(
                            error last_point.meta_data.line,
                            "expected: ;",
                            &last_point.meta_data.raw)),
                    }
                }
                (_, k) if pH!(sMatch k "condition") => {
                    stack.pop();
                    let mut last_point = points.remove(i);
                    let mut is_in_middle = false;
                    if points.len() <= i + 4 {
                        new.push(pH!(
                            error last_point.meta_data.line,
                            "bad input;",&last_point.meta_data.raw
                        ));
                        continue;
                    }

                    match (&last_point.state, stack.last()) {
                        (PointState::Token(unique_i16!(";")), Some(unique_i16!(";"))) => {
                            new.push(last_point.to_parsed());
                            continue;
                        }
                        (s, k) if pH!(hMatch s k "either" "condition") => {
                            stack.pop();
                            stack.push(unique_i16!("Or"));
                        }
                        (s, k) if pH!(hMatch s k "neither" "condition") => {
                            stack.pop();
                            stack.push(unique_i16!("nor"));
                        }
                        (s, k) if pH!(hMatch s k "both" "condition") => {
                            stack.pop();
                            stack.push(unique_i16!("And"));
                        }
                        (s, k) if pH!(hMatch s k "Or" "Or") => {
                            stack.pop();
                            is_in_middle = true;
                        }
                        (s, k) if pH!(hMatch s k "nor" "nor") => {
                            stack.pop();
                            is_in_middle = true;
                        }
                        (s, k) if pH!(hMatch s k "And" "And") => {
                            stack.pop();
                            is_in_middle = true;
                        }
                        (_) => {
                            stack.pop();
                            new.push(pH!(
                            error last_point.meta_data.line,
                            "bad input",&last_point.meta_data.raw));
                            i += 1;
                            continue;
                        }
                    }

                    new.push(last_point.to_parsed());
                    let var_point: i16;
                    match &points[i].state {
                        (PointState::Token(s)) => match token_tree[s] {
                            (Token::Var(VariableType::Integer(_))) => {
                                var_point = unique_i16!("Integer");
                            }
                            (Token::Var(VariableType::Float(_))) => {
                                var_point = unique_i16!("Float");
                            }
                            (Token::Var(VariableType::Character(_))) => {
                                var_point = unique_i16!("Character");
                            }
                            (Token::Lit(VariableType::Integer(_))) => {
                                var_point = unique_i16!("Integer");
                            }
                            (Token::Lit(VariableType::Float(_))) => {
                                var_point = unique_i16!("Float");
                            }
                            (Token::Lit(VariableType::Character(_))) => {
                                var_point = unique_i16!("Character");
                            }
                            _ => {
                                new.push(pH!(
                                        error points[i].meta_data.line,
                                        "bad input",
                                        &points[i].meta_data.raw));
                                i += 1;
                                continue;
                            }
                        },
                        _ => {
                            new.push(pH!(
                                        error points[i].meta_data.line,
                                        "bad input",
                                        &points[i].meta_data.raw));
                            i += 1;
                            continue;
                        }
                    };
                    if is_in_middle {
                        stack.push(unique_i16!("condition_end"));
                        stack.push(var_point);
                        stack.push(unique_i16!("math"));
                        stack.push(unique_i16!("operator"));
                        stack.push(var_point);
                        stack.push(unique_i16!("math"));
                    } else {
                        stack.push(unique_i16!("condition"));
                        stack.push(unique_i16!("condition_end"));
                        stack.push(var_point);
                        stack.push(unique_i16!("math"));
                        stack.push(unique_i16!("operator"));
                        stack.push(var_point);
                        stack.push(unique_i16!("math"));
                    }
                    /*stack.push(unique_i16!("condition_end"));
                    stack.push(var_point);
                    stack.push(unique_i16!("math"));
                    stack.push(unique_i16!("operator"));
                    stack.push(var_point);
                    stack.push(unique_i16!("math"));*/
                }
                (_, k) if pH!(sMatch k "condition_end") => {
                    stack.pop();
                    let mut last_point = points.remove(i);
                    match &last_point.state {
                        (PointState::Token(unique_i16!(";"))) => {
                            new.push(last_point.to_parsed());
                        }
                        (s) if pH!(hMatch s "and") || pH!(hMatch s "or") => {
                            new.push(last_point.to_parsed());
                            if points.len() <= i + 4 {
                                new.push(pH!(
                                    error last_point.meta_data.line,
                                    "bad input;",&last_point.meta_data.raw
                                ));
                                continue;
                            }
                            if points[i].state == PointState::Token(unique_i16!("not")) {
                                last_point = points.remove(i);
                                new.push(last_point.to_parsed());
                            }
                            let var_point: i16;
                            match &points[i].state {
                                (PointState::Token(s)) => match token_tree[s] {
                                    (Token::Var(VariableType::Integer(_))) => {
                                        var_point = unique_i16!("Integer");
                                    }
                                    (Token::Var(VariableType::Float(_))) => {
                                        var_point = unique_i16!("Float");
                                    }
                                    (Token::Var(VariableType::Character(_))) => {
                                        var_point = unique_i16!("Character");
                                    }
                                    (Token::Lit(VariableType::Integer(_))) => {
                                        var_point = unique_i16!("Integer");
                                    }
                                    (Token::Lit(VariableType::Float(_))) => {
                                        var_point = unique_i16!("Float");
                                    }
                                    (Token::Lit(VariableType::Character(_))) => {
                                        var_point = unique_i16!("Character");
                                    }
                                    _ => {
                                        new.push(pH!(
                                        error points[i].meta_data.line,
                                        "bad input",
                                        &points[i].meta_data.raw));
                                        i += 1;
                                        continue;
                                    }
                                },
                                _ => {
                                    if stack.last() == Some(&unique_i16!("condition")) {
                                        points.insert(i, last_point);
                                        continue;
                                    }
                                    new.push(pH!(
                                        error points[i].meta_data.line,
                                        "bad input",
                                        &points[i].meta_data.raw));
                                    i += 1;
                                    continue;
                                }
                            };
                            stack.push(unique_i16!("condition_end"));
                            stack.push(var_point);
                            stack.push(unique_i16!("math"));
                            stack.push(unique_i16!("operator"));
                            stack.push(var_point);
                            stack.push(unique_i16!("math"));
                            //i += 1;
                        }
                        _ => {
                            if stack.last() == Some(&unique_i16!("condition")) {
                                points.insert(i, last_point);
                                continue;
                            }
                            new.push(pH!(
                            error last_point.meta_data.line,
                            "expected: ;",
                            &last_point.meta_data.raw))
                        }
                    }
                }
                (s, k) if pH!(sMatch k "operator") => {
                    stack.pop();
                    match s {
                        (k) if pH!(hMatch k "LT")
                            || pH!(hMatch k "LE")
                            || pH!(hMatch k "GT")
                            || pH!(hMatch k "GE")
                            || pH!(hMatch k "NE")
                            || pH!(hMatch k "EQ") =>
                        {
                            new.push(points[i].to_parsed());
                        }
                        _ => {
                            new.push(pH!(
                                error points[i].meta_data.line,
                                "expected: LT or LE or GT or GE or NE or EQ",
                                &points[i].meta_data.raw));
                        }
                    }
                    i += 1;
                }
                _ => {
                    i += 1;
                }
            }
        }

        return (new, if stack.len() > 0 { true } else { false });
    }
}
