use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    // TODO: Implement serialization to bytes
    fn serialize(&self) -> Vec<u8>;
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        LegacyTransactionBuilder::new()
    }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        LegacyTransactionBuilder {
            version: 1,
            inputs: Vec::with_capacity(1),
            outputs: Vec::with_capacity(1),
            lock_time: 0,
        }
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        LegacyTransactionBuilder::default()
    }

    pub fn version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        self.outputs.push(output);
        self
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        self.lock_time = lock_time;
        self
    }

    pub fn build(self) -> LegacyTransaction {
        LegacyTransaction {
            version: self.version,
            inputs: self.inputs,
            outputs: self.outputs,
            lock_time: self.lock_time,
        }
    }
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    CliCommand::try_from(args)
}

#[derive(Debug)]
pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

impl TryFrom<&[String]> for CliCommand {
    type Error = BitcoinError;

    fn try_from(args: &[String]) -> Result<Self, Self::Error> {
        use BitcoinError::ParseError;

        let mut args = args.iter();
        let commands = args
            .next()
            .ok_or_else(|| ParseError("Not enough parameters".to_string()))?;

        match commands.as_str() {
            "send" => {
                let amount = args
                    .next()
                    .ok_or(ParseError("Not enough arguments".to_string()))?;
                let address = args
                    .next()
                    .ok_or(ParseError("Not enough arguments".to_string()))?
                    .to_string();

                let amount = amount
                    .parse::<u64>()
                    .map_err(|_| ParseError("Invalid amount".into()))?;

                Ok(CliCommand::Send { amount, address })
            }
            "balance" => Ok(CliCommand::Balance),
            _ => Err(BitcoinError::ParseError(format!(
                "Unknown command: {commands}"
            ))),
        }
    }
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    // Parse binary data into a LegacyTransaction
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // Minimum length is 10 bytes (4 version + 4 inputs count + 4 lock_time)
        if data.len() < 4 {
            return Err(BitcoinError::InvalidTransaction);
        }
        let version = i32::from_le_bytes(data[0..4].try_into().unwrap());

        // TODO: do we need to parse all other fields?
        Ok(LegacyTransaction::builder().version(version).build())
    }
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    // Serialize only version and lock_time (simplified)
    fn serialize(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.version.to_le_bytes());
        out.extend_from_slice(&self.lock_time.to_le_bytes());
        out
    }
}
