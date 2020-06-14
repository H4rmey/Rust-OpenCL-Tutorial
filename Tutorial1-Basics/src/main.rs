extern crate ocl;
use ocl::{Result as OclResult, Platform, Device, Context, Queue, Buffer, Program, Kernel, EventList, SpatialDims};

//string for the kernel
static SRC: &'static str = r#"
    __kernel void add(__global int *idBuffX)
    {
        int globalIdX = get_global_id(0);
        
        idBuffX[globalIdX] += + 6;
    }
"#;

const MAX_SIZE : i32 = 64;

fn main_program() -> OclResult<()> 
{
    println!("----------------main_program()----------------");

    //Get all platforms
    //Pick one platform (OpenCL in our case)
    let platforms = Platform::list();
    println!("Platforms: ");
    for platform in &platforms
    {
        println!("{}", platform);
    }
    let platform = &platforms[0];


    //Get all devices
    //Pick one device (The actual processor)    
    let devices = Device::list_all(platform);
    println!("Devices: ");
    for device in &devices
    {
        println!("{}", device[0]);
    }
    let device = devices?[0];

    //create a context for the program (using the platform ofcourse)
    let context = Context::builder().platform(*platform).build()?;

    //create a program with a source, device and context
    let program = Program::builder().src(SRC).devices(device).build(&context)?;

    //create a queue on the context and device
    let queue = Queue::new(&context, device, None)?;

    //create a buffer that can be used by the kernel
    let mut bananas = vec![1; (MAX_SIZE) as usize];
    let buff_bananas = Buffer::<i32>::builder()
        .queue(queue.clone())
        .len(bananas.len())
        .copy_host_slice(&bananas)
        .build()?;

    //create a kernel
    let kernel = Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(MAX_SIZE)
        .arg(&buff_bananas)
        .build()?;

    //enque the kernel
    unsafe
    {
        kernel.cmd().enq();
    }

    //read the values from the kernel  
    buff_bananas.read(&mut bananas).enq()?; 
    
    //print data on screen
    print!("[");
    for banana in bananas
    {
        print!("{}, ", banana);
    }
    print!("]");
    println!();

    Ok(())
}

pub fn main() 
{
    match main_program() 
    { 
        Ok(_) => (),  
        Err(err) => println!("ERROR: {}", err), 
    }
}