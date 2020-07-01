use std::{cell::RefCell, rc::Rc};
extern crate wasm_bindgen;
use wasm_bindgen::{prelude::*, JsCast};
extern crate web_sys;
//use web_sys::{Document, Element, HtmlElement, Node, Window};

#[wasm_bindgen]
pub fn greeting() -> String {
    "hello world".to_string()
}

// text単体もしくは何かしらのElementの二種類のみ
#[derive(Debug)]
enum VNodeType {
    Element,
    TextElement,
}

#[derive(Debug)]
enum NodeType {
    Div,
    Button,
}

impl NodeType {
    pub fn name(&self) -> &str {
        match self {
            Self::Div => "div",
            Self::Button => "button",
        }
    }
}

#[derive(Debug)]
struct VNode {
    vnode_type: VNodeType,
    node_type: NodeType,
    value: String,
    // TODO: eventHandlerに対応する
    attributes: Vec<(String, String)>,
    children: Vec<VNode>,
}

fn create_element(vnode: &VNode) -> web_sys::Node {
    // 受け取ったVNodeをもとに、DOMツリーを構築する
    let document = web_sys::window().unwrap().document().unwrap();
    match vnode.vnode_type {
        VNodeType::TextElement => {
            let element = document.create_text_node(&vnode.value);
            element.into()
        }
        VNodeType::Element => {
            let element = document.create_element(&vnode.node_type.name()).unwrap();
            element.set_inner_html(&vnode.value);
            //attributesの適用
            for (key, value) in &vnode.attributes {
                element.set_attribute(&key, &value);
            }
            for child in &vnode.children {
                element.append_child(&create_element(&child)).unwrap();
            }
            element.into()
        }
    }
}

// render発火用のフラグ
static should_render: bool = false;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` ");
}

#[wasm_bindgen]
pub fn render(id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let container = document.get_element_by_id(id).expect("no id `container`");

    let vnode_rc = Rc::new(RefCell::new(VNode {
        vnode_type: VNodeType::Element,
        node_type: NodeType::Div,
        value: "topノード".to_string(),
        attributes: vec![("id".to_string(), "hoge".to_string())],
        children: vec![
            VNode {
                vnode_type: VNodeType::TextElement,
                node_type: NodeType::Div,
                value: "child1だよ".to_string(),
                attributes: vec![],
                children: vec![],
            },
            VNode {
                vnode_type: VNodeType::Element,
                node_type: NodeType::Div,
                value: "child2だよ".to_string(),
                attributes: vec![],
                children: vec![VNode {
                    vnode_type: VNodeType::Element,
                    node_type: NodeType::Div,
                    value: "".to_string(),
                    attributes: vec![],
                    children: vec![
                        VNode {
                            vnode_type: VNodeType::TextElement,
                            node_type: NodeType::Div,
                            value: "child2のchild2だよ".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        VNode {
                            vnode_type: VNodeType::Element,
                            node_type: NodeType::Button,
                            value: "ボタンだよ".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                        VNode {
                            vnode_type: VNodeType::TextElement,
                            node_type: NodeType::Div,
                            value: "child2のchild2だよ".to_string(),
                            attributes: vec![],
                            children: vec![],
                        },
                    ],
                }],
            },
        ],
    }));

    let vnode = vnode_rc.clone();
    // Closureで上書きするのでNoneで良い
    let f = Rc::new(RefCell::new(None));
    // cloneで手に入るのはRefCellの参照
    let g = f.clone();
    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if i > 100 {
            // closureのcleanup
            let _ = f.borrow_mut().take();
            return;
        }
        i += 1;
        let vv = &vnode.borrow();
        // 子供があったら入れ替え、なかったらappend
        match container.first_child() {
            Some(first_child) => container
                .replace_child(&create_element(vv), &first_child)
                .unwrap(),
            None => container.append_child(&create_element(vv)).unwrap(),
        };
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));
    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
