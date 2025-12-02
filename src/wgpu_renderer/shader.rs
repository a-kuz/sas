use wgpu::*;

pub struct ShaderModule {
    pub module: wgpu::ShaderModule,
}

impl ShaderModule {
    pub fn from_wgsl(device: &Device, source: &str) -> Self {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(source.into()),
        });
        Self { module }
    }
}

