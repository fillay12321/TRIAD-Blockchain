use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::quantum::triadvantum::{
    QuantumState,
    circuit::QuantumCircuit,
    simulator::SimulationResult,
    delta::QuantumDelta,
    delta::DeltaCompressor,
    recovery::RecoveryProtocol,
    triadvantum_adapter::TriadVantumAdapter
};

pub struct VirtualNetwork {
    nodes: HashMap<String, Arc<Mutex<TriadVantumAdapter>>>,
    connections: HashMap<String, Vec<String>>,
    delta_compressor: DeltaCompressor,
    recovery_protocol: RecoveryProtocol,
}

impl VirtualNetwork {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: HashMap::new(),
            delta_compressor: DeltaCompressor::new(100),
            recovery_protocol: RecoveryProtocol::new(),
        }
    }

    pub fn add_node(&mut self, node_id: String, num_qubits: usize) -> Result<(), String> {
        if self.nodes.contains_key(&node_id) {
            return Err("Node already exists".to_string());
        }

        let adapter = TriadVantumAdapter::new(node_id.clone(), num_qubits);
        self.nodes.insert(node_id.clone(), Arc::new(Mutex::new(adapter)));
        self.connections.insert(node_id, Vec::new());
        Ok(())
    }

    pub fn connect_nodes(&mut self, from_id: &str, to_id: &str) -> Result<(), String> {
        if !self.nodes.contains_key(from_id) || !self.nodes.contains_key(to_id) {
            return Err("One or both nodes do not exist".to_string());
        }

        if let Some(connections) = self.connections.get_mut(from_id) {
            if !connections.contains(&to_id.to_string()) {
                connections.push(to_id.to_string());
            }
        }
        Ok(())
    }

    pub fn execute_circuit(&self, node_id: &str, circuit: &QuantumCircuit) -> Result<SimulationResult, String> {
        let node = self.nodes.get(node_id)
            .ok_or_else(|| "Node not found".to_string())?;
        
        let mut node = node.lock()
            .map_err(|_| "Failed to lock node".to_string())?;
        
        node.clear_circuit();
        node.add_circuit(circuit.clone());
        node.execute_circuit()
    }

    pub fn create_delta(&self, node_id: &str, from_state_id: &str, to_state_id: &str, max_size: usize) -> Result<QuantumDelta, String> {
        let node = self.nodes.get(node_id)
            .ok_or_else(|| "Node not found".to_string())?;
        
        let mut node = node.lock()
            .map_err(|_| "Failed to lock node".to_string())?;
        
        node.create_delta(from_state_id, to_state_id, max_size)
    }

    pub fn apply_delta(&self, node_id: &str, delta: &QuantumDelta, state_id: &str) -> Result<(), String> {
        let node = self.nodes.get(node_id)
            .ok_or_else(|| "Node not found".to_string())?;
        
        let mut node = node.lock()
            .map_err(|_| "Failed to lock node".to_string())?;
        
        node.apply_delta(delta, state_id)
    }

    pub fn add_checkpoint(&self, node_id: &str) -> Result<(), String> {
        let node = self.nodes.get(node_id)
            .ok_or_else(|| "Node not found".to_string())?;
        
        let mut node = node.lock()
            .map_err(|_| "Failed to lock node".to_string())?;
        
        node.add_checkpoint();
        Ok(())
    }

    pub fn recover_state(&self, node_id: &str, state_id: &str) -> Result<(), String> {
        let node = self.nodes.get(node_id)
            .ok_or_else(|| "Node not found".to_string())?;
        
        let mut node = node.lock()
            .map_err(|_| "Failed to lock node".to_string())?;
        
        node.recover_state(state_id)
    }

    pub fn get_node_state(&self, node_id: &str) -> Result<Option<QuantumState>, String> {
        let node = self.nodes.get(node_id)
            .ok_or_else(|| "Node not found".to_string())?;
        
        let node = node.lock()
            .map_err(|_| "Failed to lock node".to_string())?;
        
        Ok(node.get_state())
    }

    pub fn get_connected_nodes(&self, node_id: &str) -> Result<Vec<String>, String> {
        self.connections.get(node_id)
            .cloned()
            .ok_or_else(|| "Node not found".to_string())
    }
} 