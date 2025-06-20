//! HAF Translation Contracts
//! 
//! Explicit contracts for data transformation between layers.
//! These contracts make layer boundaries clear and testable.

use super::*;

/// Contract for VNode to DOM operations (L1 → L2)
pub struct VNodeToDomContract;

#[derive(Clone, Debug)]
pub enum DomOp {
    CreateElement { tag: String, id: usize },
    CreateText { text: String, id: usize },
    SetAttribute { id: usize, name: String, value: String },
    RemoveAttribute { id: usize, name: String },
    AppendChild { parent: usize, child: usize },
    RemoveChild { parent: usize, child: usize },
    ReplaceChild { parent: usize, old: usize, new: usize },
    UpdateText { id: usize, text: String },
}

impl L1ToL2Contract for VNodeToDomContract {
    type L1Type = VNode;
    type L2Type = Vec<DomOp>;
    
    fn translate(vnode: Self::L1Type) -> Self::L2Type {
        let mut ops = Vec::new();
        let mut next_id = 0;
        
        fn build_ops(node: &VNode, ops: &mut Vec<DomOp>, id: &mut usize) -> usize {
            let node_id = *id;
            *id += 1;
            
            match node {
                VNode::Text(text) => {
                    ops.push(DomOp::CreateText {
                        text: text.clone(),
                        id: node_id,
                    });
                }
                VNode::Element { tag, props, children } => {
                    ops.push(DomOp::CreateElement {
                        tag: tag.clone(),
                        id: node_id,
                    });
                    
                    for (name, value) in &props.attributes {
                        ops.push(DomOp::SetAttribute {
                            id: node_id,
                            name: name.clone(),
                            value: value.clone(),
                        });
                    }
                    
                    for child in children {
                        let child_id = build_ops(child, ops, id);
                        ops.push(DomOp::AppendChild {
                            parent: node_id,
                            child: child_id,
                        });
                    }
                }
            }
            
            node_id
        }
        
        build_ops(&vnode, &mut ops, &mut next_id);
        ops
    }
}

/// Contract for DOM operations to HTTP response (L2 → L3)
pub struct DomToHttpContract;

#[derive(Clone, Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[cfg(feature = "ssr")]
impl axum::response::IntoResponse for HttpResponse {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        
        let mut response = axum::response::Response::builder()
            .status(StatusCode::from_u16(self.status).unwrap_or(StatusCode::OK));
        
        for (name, value) in self.headers {
            response = response.header(name, value);
        }
        
        response.body(self.body.into()).unwrap()
    }
}

impl L2ToL3Contract for DomToHttpContract {
    type L2Type = String; // Rendered HTML
    type L3Type = HttpResponse;
    
    fn translate(html: Self::L2Type) -> Self::L3Type {
        HttpResponse {
            status: 200,
            headers: vec![
                ("Content-Type".to_string(), "text/html".to_string()),
                ("X-Powered-By".to_string(), "Layer9-HAF".to_string()),
            ],
            body: format!(
                "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>{}</body></html>",
                html
            ),
        }
    }
}

/// Contract for Signal changes to Effects (L1 → L2)
pub struct SignalToEffectContract;

#[derive(Clone, Debug)]
pub struct SignalChange {
    pub signal_id: usize,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub effect_id: usize,
    pub dependencies: Vec<usize>,
    pub action: EffectAction,
}

#[derive(Clone, Debug)]
pub enum EffectAction {
    UpdateDom { target: usize },
    TriggerRender { component: usize },
    RunCallback { callback_id: usize },
}

impl L1ToL2Contract for SignalToEffectContract {
    type L1Type = SignalChange;
    type L2Type = Vec<Effect>;
    
    fn translate(change: Self::L1Type) -> Self::L2Type {
        // In a real implementation, this would look up registered effects
        vec![
            Effect {
                effect_id: 1,
                dependencies: vec![change.signal_id],
                action: EffectAction::TriggerRender { component: 0 },
            }
        ]
    }
}

/// Contract for Route definitions to HTTP handlers (L1 → L3)
pub struct RouteToHandlerContract;

#[derive(Clone, Debug)]
pub struct Route {
    pub path: String,
    pub method: String,
    pub component: String,
}

pub type Handler = Box<dyn Fn(&str) -> HttpResponse>;

impl L1ToL2Contract for RouteToHandlerContract {
    type L1Type = Route;
    type L2Type = Handler;
    
    fn translate(route: Self::L1Type) -> Self::L2Type {
        Box::new(move |_path| {
            HttpResponse {
                status: 200,
                headers: vec![],
                body: format!("Route: {} {}", route.method, route.path),
            }
        })
    }
}

/// Batch contract for multiple transformations
pub struct BatchContract<C: L1ToL2Contract> {
    _phantom: PhantomData<C>,
}

impl<C: L1ToL2Contract> L1ToL2Contract for BatchContract<C> {
    type L1Type = Vec<C::L1Type>;
    type L2Type = Vec<C::L2Type>;
    
    fn translate(items: Self::L1Type) -> Self::L2Type {
        items.into_iter().map(C::translate).collect()
    }
}

/// Contract composition
pub struct ComposedContract<C1, C2> 
where
    C1: L1ToL2Contract,
    C2: L2ToL3Contract<L2Type = C1::L2Type>,
{
    _c1: PhantomData<C1>,
    _c2: PhantomData<C2>,
}

impl<C1, C2> ComposedContract<C1, C2>
where
    C1: L1ToL2Contract,
    C2: L2ToL3Contract<L2Type = C1::L2Type>,
{
    pub fn translate_full(l1: C1::L1Type) -> C2::L3Type {
        let l2 = C1::translate(l1);
        C2::translate(l2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vnode_to_dom_contract() {
        let vnode = VNode::Element {
            tag: "div".to_string(),
            props: Props {
                attributes: vec![("class".to_string(), "container".to_string())],
            },
            children: vec![
                VNode::Text("Hello, HAF!".to_string()),
            ],
        };
        
        let ops = VNodeToDomContract::translate(vnode);
        
        assert_eq!(ops.len(), 4); // create div, set class, create text, append child
    }
    
    #[test]
    fn test_contract_composition() {
        let vnode = VNode::Text("Hello".to_string());
        let ops = VNodeToDomContract::translate(vnode);
        // Further transformations would happen here
        assert!(!ops.is_empty());
    }
}