extern crate ocl;
use ocl::{Result as OclResult, Platform, Device, Context, Queue, Buffer, Program, Kernel, EventList, SpatialDims};

//string for the kernel
static SRC: &'static str = r#"
    __kernel void add(__global int *buff_x, __global int *buff_y)
    {
        int global_id_x = get_local_id(0);

        buff_x[global_id_x] += buff_y[global_id_x];
    }
"#;

const MAX_SIZE : usize = 8;

fn main_program() -> OclResult<()> 
{
    println!("----------------main_program()----------------");

    //Get all platforms
    //Pick one platform (OpenCL in our case)
    let platforms = Platform::list();
    println!("Platforms: ");
    for platform in &platforms
    {
        //println!("{}", platform);
    }
    let platform = &platforms[0];


    //Get all devices
    //Pick one device (The actual processor)    
    let devices = Device::list_all(platform);
    println!("Devices: ");
    for device in &devices
    {
        //println!("{}", device[0]);
    }
    let device = devices?[0];

    //create a context for the program (using the platform ofcourse)
    let context = Context::builder().platform(*platform).build()?;

    //create a program with a source, device and context
    let program = Program::builder().src(SRC).devices(device).build(&context)?;

    //create a queue on the context and device
    let queue = Queue::new(&context, device, None)?;

    //create a buffer that can be used by the kernel
    let mut bananas = vec![0; MAX_SIZE * MAX_SIZE];
    for x in 0..MAX_SIZE
    {
        bananas[x] = x as i32;
    }
    let buff_bananas = Buffer::<i32>::builder()
        .queue(queue.clone())
        .len(bananas.len())
        .copy_host_slice(&bananas)
        .build()?;

    let mut apples = vec![0; MAX_SIZE * MAX_SIZE];
    for x in 0..MAX_SIZE
    {
        apples[x] = x as i32;  
    }
    let buff_apples = Buffer::<i32>::builder()
        .queue(queue.clone())
        .len(apples.len())
        .copy_host_slice(&apples)
        .build()?;

    //create a kernel
    let kernel = Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(MAX_SIZE)
        .arg(&buff_bananas)
        .arg(&buff_apples)
        .build()?;

    //enque the kernel
    unsafe
    {
        kernel.cmd().enq();
    }

    //read the values from the kernel  
    buff_bananas.read(&mut bananas).enq()?; 
    buff_apples.read(&mut apples).enq()?; 
    
    //print data on screen    
    for x in 0..MAX_SIZE
    {
        print!("|{}|\t", bananas[x]);
    } 
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