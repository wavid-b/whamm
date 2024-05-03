use std::collections::HashMap;
use crate::parser::types as parser_types;
use parser_types::{DataType, Whammy, Whamm, WhammVisitor, Expr, Fn, Function, Module, Op, Probe, Provider, Statement, Value};
use crate::verifier::types::{Record, ScopeType, SymbolTable};

use log::{error, trace};

pub struct SymbolTableBuilder {
    pub table: SymbolTable,

    // TODO -- these should be updated as they are entered/exited
    curr_whamm: Option<usize>,   // indexes into this::table::records
    curr_whammy: Option<usize>,  // indexes into this::table::records
    curr_provider: Option<usize>, // indexes into this::table::records
    curr_module: Option<usize>,   // indexes into this::table::records
    curr_function: Option<usize>, // indexes into this::table::records
    curr_probe: Option<usize>,    // indexes into this::table::records

    curr_fn: Option<usize>,       // indexes into this::table::records
}
impl SymbolTableBuilder {
    pub fn new() -> Self {
        SymbolTableBuilder {
            table: SymbolTable::new(),
            curr_whamm: None,
            curr_whammy: None,
            curr_provider: None,
            curr_module: None,
            curr_function: None,
            curr_probe: None,
            curr_fn: None,
        }
    }

    fn add_whammy(&mut self, whammy: &Whammy) {
        if self.table.lookup(&whammy.name).is_some() {
            error!("duplicated whammy [ {} ]", &whammy.name);
        }

        // create record
        let whammy_rec = Record::Whammy {
            name: whammy.name.clone(),
            fns: vec![],
            globals: vec![],
            providers: vec![],
        };

        // Add whammy to scope
        let id = self.table.put(whammy.name.clone(), whammy_rec);

        // Add whammy to current whamm record
        match self.table.get_record_mut(&self.curr_whamm.unwrap()).unwrap() {
            Record::Whamm { whammys, .. } => {
                whammys.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }

        // enter whammy scope
        self.table.enter_scope();
        self.curr_whammy = Some(id.clone());

        // set scope name and type
        self.table.set_curr_scope_info(whammy.name.clone(), ScopeType::Whammy);
        self.table.set_curr_whammy(id.clone());
    }

    fn add_provider(&mut self, provider: &Provider) {
        if self.table.lookup(&provider.name).is_some() {
            error!("duplicated provider [ {} ]", &provider.name);
        }

        // create record
        let provider_rec = Record::Provider {
            name: provider.name.clone(),
            fns: vec![],
            globals: vec![],
            modules: vec![],
        };

        // Add provider to scope
        let id = self.table.put(provider.name.clone(), provider_rec);

        // Add provider to current whammy record
        match self.table.get_record_mut(&self.curr_whammy.unwrap()).unwrap() {
            Record::Whammy { providers, .. } => {
                providers.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }

        // enter provider scope
        self.table.enter_scope();
        self.curr_provider = Some(id.clone());

        // set scope name and type
        self.table.set_curr_scope_info(provider.name.clone(), ScopeType::Provider);
    }

    fn add_module(&mut self, module: &Module) {
        if self.table.lookup(&module.name).is_some() {
            error!("duplicated module [ {} ]", &module.name);
        }

        // create record
        let module_rec = Record::Module {
            name: module.name.clone(),
            fns: vec![],
            globals: vec![],
            functions: vec![],
        };

        // Add module to scope
        let id = self.table.put(module.name.clone(), module_rec);

        // Add module to current provider record
        match self.table.get_record_mut(&self.curr_provider.unwrap()).unwrap() {
            Record::Provider { modules, .. } => {
                modules.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }

        // enter module scope
        self.table.enter_scope();
        self.curr_module = Some(id.clone());

        // set scope name and type
        self.table.set_curr_scope_info(module.name.clone(), ScopeType::Module);
    }

    fn add_function(&mut self, function: &Function) {
        if self.table.lookup(&function.name).is_some() {
            error!("duplicated function [ {} ]", &function.name);
        }

        // create record
        let function_rec = Record::Function {
            name: function.name.clone(),
            fns: vec![],
            globals: vec![],
            probes: vec![],
        };

        // Add function to scope
        let id = self.table.put(function.name.clone(), function_rec);

        // Add function to current module record
        match self.table.get_record_mut(&self.curr_module.unwrap()).unwrap() {
            Record::Module { functions, .. } => {
                functions.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }

        // enter function scope
        self.table.enter_scope();
        self.curr_function = Some(id.clone());

        // set scope name and type
        self.table.set_curr_scope_info(function.name.clone(), ScopeType::Function);
    }

    fn add_probe(&mut self, probe: &Probe) {
        if self.table.lookup(&probe.name).is_some() {
            error!("duplicated probe [ {} ]", &probe.name);
        }

        // create record
        let probe_rec = Record::Probe {
            name: probe.name.clone(),
            fns: vec![],
            globals: vec![],
        };

        // Add probe to scope
        let id = self.table.put(probe.name.clone(), probe_rec);

        // Add probe to current function record
        match self.table.get_record_mut(&self.curr_function.unwrap()) {
            Some(Record::Function { probes, .. }) => {
                probes.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }

        // enter probe scope
        self.table.enter_scope();
        self.curr_probe = Some(id.clone());

        // set scope name and type
        self.table.set_curr_scope_info(probe.name.clone(), ScopeType::Probe);
    }

    fn add_fn(&mut self, f: &Fn) {
        if self.table.lookup(&f.name).is_some() {
            error!("duplicated fn [ {} ]", &f.name);
        }

        // create record
        let fn_rec = Record::Fn {
            name: f.name.clone(),
            params: vec![],
            addr: None
        };

        // Add fn to scope
        let id = self.table.put(f.name.clone(), fn_rec);

        // add fn record to the current record
        self.add_fn_id_to_curr_rec(id);

        // enter fn scope
        self.table.enter_scope();
        self.curr_fn = Some(id.clone());

        // set scope name and type
        self.table.set_curr_scope_info(f.name.clone(), ScopeType::Fn);

        // visit parameters
        f.params.iter().for_each(| param | self.visit_formal_param(param));
    }

    fn add_fn_id_to_curr_rec(&mut self, id: usize) {
        match self.table.get_curr_rec_mut() {
            Some(Record::Whamm { fns, .. }) |
            Some(Record::Whammy { fns, .. }) |
            Some(Record::Provider { fns, .. }) |
            Some(Record::Module { fns, .. }) |
            Some(Record::Function { fns, .. }) |
            Some(Record::Probe { fns, .. }) => {
                fns.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn add_param(&mut self, var_id: &Expr, ty: &DataType) {
        let name = match var_id {
            Expr::VarId {name} => name,
            _ => {
                unreachable!();
            }
        };

        // create record
        let param_rec = Record::Var {
            name: name.clone(),
            ty: ty.clone(),
            value: None,
            addr: None
        };

        // add var to scope
        let id = self.table.put(name.clone(), param_rec);

        // add param to fn record
        match self.table.get_record_mut(&self.curr_fn.unwrap()) {
            Some(Record::Fn { params, .. }) => {
                params.push(id.clone());
            }
            _ => {
                unreachable!()
            }
        }
    }

    /// Insert `global` record into scope
    fn add_global(&mut self, ty: DataType, name: String) {
        if self.table.lookup(&name).is_some() {
            error!("duplicated identifier [ {} ]", name);
        }

        // Add global to scope
        let id = self.table.put(name.clone(), Record::Var {
            ty,
            name,
            value: None,
            addr: None
        });

        // add global record to the current record
        self.add_fn_id_to_curr_rec(id);
    }

    fn visit_globals(&mut self, globals: &HashMap<String, (DataType, Expr, Option<Value>)>) {
        for (name, (ty, _expr, _val)) in globals.iter() {
            self.add_global(ty.clone(), name.clone());
        }
    }
}

impl WhammVisitor<()> for SymbolTableBuilder {
    fn visit_whamm(&mut self, whamm: &Whamm) -> () {
        trace!("Entering: visit_whamm");
        let name: String = "whamm".to_string();
        self.table.set_curr_scope_info(name.clone(), ScopeType::Whamm);

        // add whamm record
        let whamm_rec = Record::Whamm {
            name: name.clone(),
            fns: vec![],
            globals: vec![],
            whammys: vec![],
        };

        // Add whamm to scope
        let id = self.table.put(name.clone(), whamm_rec);

        self.curr_whamm = Some(id);

        // visit fns
        whamm.fns.iter().for_each(| f | self.visit_fn(f) );

        // visit globals
        self.visit_globals(&whamm.globals);

        // visit whammys
        whamm.whammys.iter().for_each(| whammy | self.visit_whammy(whammy));

        trace!("Exiting: visit_whamm");
        self.curr_whamm = None;
    }

    fn visit_whammy(&mut self, whammy: &Whammy) -> () {
        trace!("Entering: visit_whammy");

        self.add_whammy(whammy);
        whammy.fns.iter().for_each(| f | self.visit_fn(f) );
        self.visit_globals(&whammy.globals);
        whammy.providers.iter().for_each(| (_name, provider) | {
            self.visit_provider(provider)
        });

        trace!("Exiting: visit_whammy");
        self.table.exit_scope();
        self.curr_whammy = None;
    }

    fn visit_provider(&mut self, provider: &Provider) -> () {
        trace!("Entering: visit_provider");

        self.add_provider(provider);
        provider.fns.iter().for_each(| f | self.visit_fn(f) );
        self.visit_globals(&provider.globals);
        provider.modules.iter().for_each(| (_name, module) | {
            self.visit_module(module)
        });

        trace!("Exiting: visit_provider");
        self.table.exit_scope();
        self.curr_provider = None;
    }

    fn visit_module(&mut self, module: &Module) -> () {
        trace!("Entering: visit_module");

        self.add_module(module);
        module.fns.iter().for_each(| f | self.visit_fn(f) );
        self.visit_globals(&module.globals);
        module.functions.iter().for_each(| (_name, function) | {
            self.visit_function(function)
        });

        trace!("Exiting: visit_module");
        self.table.exit_scope();
        self.curr_module = None;
    }

    fn visit_function(&mut self, function: &Function) -> () {
        trace!("Entering: visit_function");

        self.add_function(function);
        function.fns.iter().for_each(| f | self.visit_fn(f) );
        self.visit_globals(&function.globals);

        // visit probe_map
        function.probe_map.iter().for_each(| probes | {
            probes.1.iter().for_each(| probe | {
                self.visit_probe(probe);
            });
        });

        trace!("Exiting: visit_function");
        self.table.exit_scope();
        self.curr_function = None;
    }

    fn visit_probe(&mut self, probe: &Probe) -> () {
        trace!("Entering: visit_probe");

        self.add_probe(probe);
        probe.fns.iter().for_each(| f | self.visit_fn(f) );
        self.visit_globals(&probe.globals);

        // Will not visit predicate/body at this stage

        trace!("Exiting: visit_probe");
        self.table.exit_scope();
        self.curr_probe = None;
    }

    fn visit_fn(&mut self, f: &Fn) -> () {
        trace!("Entering: visit_fn");

        // add fn
        self.add_fn(f);

        // Will not visit predicate/body at this stage

        trace!("Exiting: visit_fn");
        self.table.exit_scope();
        self.curr_fn = None;
    }

    fn visit_formal_param(&mut self, param: &(Expr, DataType)) -> () {
        trace!("Entering: visit_formal_param");

        // add param
        self.add_param(&param.0, &param.1);

        trace!("Exiting: visit_formal_param");
    }

    fn visit_stmt(&mut self, _assign: &Statement) -> () {
        // Not visiting function/probe bodies
        unreachable!()
    }

    fn visit_expr(&mut self, _call: &Expr) -> () {
        // Not visiting predicates/statements
        unreachable!()
    }

    fn visit_op(&mut self, _op: &Op) -> () {
        // Not visiting predicates/statements
        unreachable!()
    }

    fn visit_datatype(&mut self, _datatype: &DataType) -> () {
        // Not visiting predicates/statements
        unreachable!()
    }

    fn visit_value(&mut self, _val: &Value) -> () {
        // Not visiting predicates/statements
        unreachable!()
    }
}