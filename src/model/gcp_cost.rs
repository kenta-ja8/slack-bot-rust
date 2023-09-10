use std::collections::HashMap;

#[derive(Debug)]
pub struct CostReport {
    pub diff_rate: Option<f64>,
    pub diff_cost: f64,
    pub cost: f64,
}


pub type ServiceToCostReportMap = HashMap<String, CostReport>;
