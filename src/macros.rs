macro_rules! ok {
    ( $expr:expr ) => {
        $expr;
        return Ok(());
    };
}

macro_rules! route {
    ( $name:ident, $req:ident, $res:ident, $ctx:ident, $body:expr) => {
        #[allow(unused_mut, unused_variables)]
        pub fn $name($req: &mut ::hayaku::Request, $res: &mut ::hayaku::Response, $ctx: &::Context)
            -> ::Result<()>
        {
            $body
        }
    };
}

macro_rules! redirect {
    ( $res:ident, $ctx:ident, $path:expr, $msg:expr) => {
        ok!($res.redirect(::hayaku::Status::FOUND, &format!("{}{}", $ctx.mount, $path), $msg));
    };
}

macro_rules! error {
    ( $res:ident, $ctx:ident, $path:expr, $msg:expr ) => {
        //::routes::util::create_error($res, $ctx, $msg.to_string());
        redirect!($res, $ctx, $path, $msg);
    };
}

macro_rules! check_login {
    ( $cookies:expr, $res:ident, $ctx:ident ) => {
        {
            if let Some(name) = ::routes::util::check_login($ctx, $cookies)? {
                name
            } else {
                error!($res, $ctx, "", "You must be logged in for this");
            }
        }
    };
}

macro_rules! parse_param {
    ( $req:ident, $res:ident, $ctx:ident, $name:expr, $t:ty) => {
        {
            match $req.get_param($name).parse::<$t>() {
                Ok(p) => p,
                Err(_) => return ::routes::not_found($req, $res, $ctx),
            }
        }
    };
}

macro_rules! tmpl {
    ( $req:ident, $res:ident, $ctx:ident, $name:expr, $body:expr ) => {
        //let err = ::routes::util::get_error($req, $res);
        //let tmpl = ::templates::Template::new($ctx, $name, $body);
        let tmpl = ::templates::Template::new($name, $body);
        let headers = $req.headers();
        let compress = headers.get(::hayaku::header::ACCEPT_ENCODING);

        if compress.is_some() {
            let compress = compress.unwrap().to_str().unwrap();
            if compress.contains("br") {
                $res.add_header(::hayaku::header::CONTENT_ENCODING,
                                ::hayaku::header::HeaderValue::from_static("br"));
                let mut encoder = ::brotli::CompressorWriter::new($res, 4096, 4, 22);
                use std::io::Write;
                write!(&mut encoder, "{}", tmpl)?;
                encoder.flush()?;
                return Ok(());
            } else if compress.contains("gzip") {
                $res.add_header(::hayaku::header::CONTENT_ENCODING,
                                ::hayaku::header::HeaderValue::from_static("gzip"));
                let mut encoder = ::libflate::gzip::Encoder::new($res)?;
                use std::io::Write;
                write!(&mut encoder, "{}", tmpl)?;
                encoder.finish().into_result()?;
                return Ok(());
            }
        }
        ok!($res.fmt_body(tmpl));
    };
}
