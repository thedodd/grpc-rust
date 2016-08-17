use marshall::*;


pub struct MethodDescriptor<Req, Resp> {
    pub name: String,
    pub input_streaming: bool,
    pub output_streaming: bool,
    pub req_marshaller: Box<Marshaller<Req>>,
    pub resp_marshaller: Box<Marshaller<Resp>>,
}

pub trait MethodHandler<Req, Resp> {
    fn handle(&self, req: Req) -> Resp;
}

pub struct MethodHandlerEcho;

impl<A> MethodHandler<A, A> for MethodHandlerEcho {
    fn handle(&self, req: A) -> A {
        println!("handle echo");
        req
    }
}

struct MethodHandlerFn<F> {
    f: F
}

impl<Req, Resp, F : Fn(Req) -> Resp> MethodHandler<Req, Resp> for MethodHandlerFn<F> {
    fn handle(&self, req: Req) -> Resp {
        (self.f)(req)
    }
}

trait MethodHandlerDispatch {
    fn on_message(&self, message: &[u8]) -> Vec<u8>;
}

struct MethodHandlerDispatchImpl<Req, Resp> {
    desc: MethodDescriptor<Req, Resp>,
    method_handler: Box<MethodHandler<Req, Resp>>,
}

impl<Req, Resp> MethodHandlerDispatch for MethodHandlerDispatchImpl<Req, Resp> {
    fn on_message(&self, message: &[u8]) -> Vec<u8> {
        let req = self.desc.req_marshaller.read(message);
        let resp = self.method_handler.handle(req);
        self.desc.resp_marshaller.write(&resp)
    }
}

pub struct ServerMethod {
    name: String,
    dispatch: Box<MethodHandlerDispatch>,
}

impl ServerMethod {
    pub fn new<Req : 'static, Resp : 'static>(method: MethodDescriptor<Req, Resp>, handler: Box<MethodHandler<Req, Resp>>) -> ServerMethod {
        ServerMethod {
            name: method.name.clone(),
            dispatch: Box::new(MethodHandlerDispatchImpl {
                desc: method,
                method_handler: handler,    
            }),
        }
    }
}

pub struct ServerServiceDefinition {
    methods: Vec<ServerMethod>,
}

impl ServerServiceDefinition {
    pub fn new(mut methods: Vec<ServerMethod>) -> ServerServiceDefinition {
        methods.push(
            ServerMethod::new(
                MethodDescriptor {
                    name: "/helloworld.Greeter/SayHello".to_owned(),
                    input_streaming: false,
                    output_streaming: false,
                    req_marshaller: Box::new(MarshallerBytes),
                    resp_marshaller: Box::new(MarshallerBytes),
                },
                Box::new(MethodHandlerEcho)
            )
        );
        ServerServiceDefinition {
            methods: methods,
        }
    }

    pub fn find_method(&self, name: &str) -> &ServerMethod {
        self.methods.iter()
            .filter(|m| m.name == name)
            .next()
            .expect(&format!("unknown method: {}", name))
    }

    pub fn handle_method(&self, name: &str, message: &[u8]) -> Vec<u8> {
        self.find_method(name).dispatch.on_message(message)
    }
}
