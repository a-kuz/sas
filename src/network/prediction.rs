use super::PACKET_BACKUP;

pub const CMD_BACKUP: usize = 64;

#[derive(Clone, Copy, Debug)]
pub struct UserCommand {
    pub server_time: u32,
    pub sequence: u32,
    pub move_forward: f32,
    pub move_right: f32,
    pub buttons: u32,
    pub angles: f32,
}

impl Default for UserCommand {
    fn default() -> Self {
        Self {
            server_time: 0,
            sequence: 0,
            move_forward: 0.0,
            move_right: 0.0,
            buttons: 0,
            angles: 0.0,
        }
    }
}

pub struct CommandBuffer {
    commands: [UserCommand; CMD_BACKUP],
    current_cmd_number: u32,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            commands: [UserCommand::default(); CMD_BACKUP],
            current_cmd_number: 0,
        }
    }
    
    pub fn add_command(&mut self, cmd: UserCommand) -> u32 {
        let sequence = self.current_cmd_number;
        let index = (sequence % CMD_BACKUP as u32) as usize;
        
        self.commands[index] = UserCommand {
            sequence,
            ..cmd
        };
        
        self.current_cmd_number = self.current_cmd_number.wrapping_add(1);
        sequence
    }
    
    pub fn get_command(&self, sequence: u32) -> Option<UserCommand> {
        let age = self.current_cmd_number.wrapping_sub(sequence);
        if age >= CMD_BACKUP as u32 {
            return None;
        }
        
        let index = (sequence % CMD_BACKUP as u32) as usize;
        Some(self.commands[index])
    }
    
    pub fn get_commands_since(&self, since_sequence: u32) -> Vec<UserCommand> {
        let mut cmds = Vec::new();
        let current = self.current_cmd_number;
        
        for seq in since_sequence..current {
            if let Some(cmd) = self.get_command(seq) {
                cmds.push(cmd);
            }
        }
        
        cmds
    }
    
    pub fn current_sequence(&self) -> u32 {
        self.current_cmd_number
    }
}

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_buffer_add() {
        let mut buffer = CommandBuffer::new();
        
        let cmd = UserCommand {
            move_right: 1.0,
            buttons: 1,
            ..Default::default()
        };
        
        let seq = buffer.add_command(cmd);
        assert_eq!(seq, 0);
        
        let retrieved = buffer.get_command(seq).unwrap();
        assert_eq!(retrieved.sequence, 0);
        assert_eq!(retrieved.move_right, 1.0);
    }
    
    #[test]
    fn test_command_buffer_wraparound() {
        let mut buffer = CommandBuffer::new();
        
        for i in 0..CMD_BACKUP + 10 {
            let cmd = UserCommand {
                move_right: i as f32,
                ..Default::default()
            };
            buffer.add_command(cmd);
        }
        
        let old_cmd = buffer.get_command(0);
        assert!(old_cmd.is_none());
        
        let recent_cmd = buffer.get_command((CMD_BACKUP + 9) as u32);
        assert!(recent_cmd.is_some());
    }
    
    #[test]
    fn test_get_commands_since() {
        let mut buffer = CommandBuffer::new();
        
        for i in 0..10 {
            let cmd = UserCommand {
                move_right: i as f32,
                ..Default::default()
            };
            buffer.add_command(cmd);
        }
        
        let cmds = buffer.get_commands_since(5);
        assert_eq!(cmds.len(), 5);
        assert_eq!(cmds[0].sequence, 5);
        assert_eq!(cmds[4].sequence, 9);
    }
}











