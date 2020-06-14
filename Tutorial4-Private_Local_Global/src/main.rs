extern crate ocl;
use ocl::{Result as OclResult, Platform, Device, Context, Queue, Buffer, Program, Kernel, EventList, SpatialDims};

//string for the kernel
static SRC: &'static str = r#"
    __kernel void add(__global int *in, __global int *out, __local int *temp)
    {        
        int sum;
        uint global_id;

        global_id = get_global_id(0) * 2;

        temp[global_id/2] = in[global_id] + in[global_id+1]; 

        barrier(CLK_LOCAL_MEM_FENCE);

        if(get_global_id(0) == 0) 
        {
            sum = 0;
            for(int i=0; i < get_global_size(0)/2; i+=1) 
            {
                sum += temp[i];
            }
            out[0] = sum;
        }
    }
"#;

const MAX_SIZE : usize = 8;

fn create_countup_vector(size : usize, doDebug : bool) -> Vec<i32>
{    
    let mut temp = vec![0; MAX_SIZE * MAX_SIZE];
    for x in 0..MAX_SIZE
    {
            temp[x] = x as i32;
    }

    if doDebug
    {
        print_vector(&temp, MAX_SIZE);
    }

    temp
}

fn print_vector(vector : &Vec<i32>, size : usize)
{
    println!("-----------------------------------------------------------------");
    for x in 0..size
    {
        print!("|{}|\t", vector[x]);
    } 
    println!();
}

fn create_countup_vector2d(size : usize, doDebug : bool) -> Vec<i32>
{    
    let mut temp = vec![0; MAX_SIZE * MAX_SIZE];
    for x in 0..MAX_SIZE
    {
        for y in 0..MAX_SIZE
        {
            temp[(MAX_SIZE * x + y)] = (MAX_SIZE * x + y) as i32;
        }
    }

    if doDebug
    {
        print_vector2d(&temp, MAX_SIZE, MAX_SIZE)
    }

    temp
}

fn print_vector2d(vector : &Vec<i32>, width : usize, height : usize)
{
    println!("-----------------------------------------------------------------");
    for x in 0..width
    {
        for y in 0..height
        {
            print!("|{}|\t", vector[(MAX_SIZE * x + y)]);
        }
        println!();
    } 
    println!();
}

fn main_program() -> OclResult<()> 
{
    println!("----------------main_program()----------------");

    //------------------------setup------------------------//
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

    //------------------------main program------------------------//
    //create a buffer that can be used by the kernel
    let mut bananas = create_countup_vector(MAX_SIZE, true);
    let buff_bananas = Buffer::<i32>::builder().queue(queue.clone()).len(bananas.len()).copy_host_slice(&bananas).build()?;

    let mut apples = vec![0; 1];
    let buff_apples = Buffer::<i32>::builder().queue(queue.clone()).len(apples.len()).copy_host_slice(&apples).build()?;
    
    //create a kernel
    let salad = Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(MAX_SIZE)
        .arg(&buff_bananas)
        .arg(&buff_apples)
        .arg(None::<&Buffer<i32>>)
        .build()?;

    salad.set_arg(2, None::<&Buffer<i32>>);

    //enque the kernel
    unsafe
    {
        salad.cmd().enq();
    }

    //-----------------end of program-------------------//
    //read the values from the kernel  
    buff_bananas.read(&mut bananas).enq()?; 
    buff_apples.read(&mut apples).enq()?; 
    
    //print data on screen    
    println!("{}", apples[0]);

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