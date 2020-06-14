extern crate ocl;
use ocl::{Result as OclResult, Platform, Device, Context, Queue, Buffer, Program, Kernel, EventList, SpatialDims};

//string for the kernel
static SRC: &'static str = r#"
    __kernel void add(__global int *bananas, __global int *apples)
    {  
        bananas[get_local_id(0)] = get_global_id(0);
        apples[get_global_id(0)] = get_local_id(0);
    }
"#;

const GLOBAL_WORK_SIZE : usize = 24;
const LOCAL_WORK_SIZE : usize = 4;

fn create_countup_vector(size : usize, doDebug : bool) -> Vec<i32>
{    
    let mut temp = vec![0; GLOBAL_WORK_SIZE * GLOBAL_WORK_SIZE];
    for x in 0..GLOBAL_WORK_SIZE
    {
            temp[x] = x as i32;
    }

    if doDebug
    {
        print_vector(&temp, GLOBAL_WORK_SIZE);
    }

    temp
}

fn print_vector(vector : &Vec<i32>, size : usize)
{
    println!("-----------------------------------------------------------------");
    for x in 0..size
    {
        print!(" {} ", vector[x]);
    } 
    println!();
    println!("-----------------------------------------------------------------");
}

fn create_countup_vector2d(size : usize, doDebug : bool) -> Vec<i32>
{    
    let mut temp = vec![0; GLOBAL_WORK_SIZE * GLOBAL_WORK_SIZE];
    for x in 0..GLOBAL_WORK_SIZE
    {
        for y in 0..GLOBAL_WORK_SIZE
        {
            temp[(GLOBAL_WORK_SIZE * x + y)] = (GLOBAL_WORK_SIZE * x + y) as i32;
        }
    }

    if doDebug
    {
        print_vector2d(&temp, GLOBAL_WORK_SIZE, GLOBAL_WORK_SIZE)
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
            print!("|{}|\t", vector[ (height * x + y)]);
        }
        println!();
    } 
    println!("-----------------------------------------------------------------");
}

fn main_program() -> OclResult<()> 
{
    println!("----------------main_program()----------------");

    //------------------------setup------------------------//
    //Get all platforms
    //Pick one platform (OpenCL in our case)
    let platforms = Platform::list();
    let platform = &platforms[0];

    //Get all devices
    //Pick one device (The actual processor)    
    let devices = Device::list_all(platform);
    let device = devices?[0];

    //create a context for the program (using the platform ofcourse)
    let context = Context::builder().platform(*platform).build()?;

    //create a program with a source, device and context
    let program = Program::builder().src(SRC).devices(device).build(&context)?;

    //create a queue on the context and device
    let queue = Queue::new(&context, device, None)?;

    //------------------------main program------------------------//
    //create a buffer that can be used by the kernel
    let mut bananas = vec![0; GLOBAL_WORK_SIZE * LOCAL_WORK_SIZE];
    let buff_bananas = Buffer::<i32>::builder().queue(queue.clone()).len(bananas.len()).copy_host_slice(&bananas).build()?;

    let mut apples = vec![0; GLOBAL_WORK_SIZE];
    let buff_apples = Buffer::<i32>::builder().queue(queue.clone()).len(apples.len()).copy_host_slice(&apples).build()?;
    
    //create a kernel
    let salad = Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(GLOBAL_WORK_SIZE)
        .local_work_size(LOCAL_WORK_SIZE)
        .arg(&buff_bananas)
        .arg(&buff_apples)
        .build()?;


    //enque the kernel
    unsafe
    {
        salad.cmd().enq()?;
    }

    //-----------------end of program-------------------//
    //read the values from the kernel  
    buff_bananas.read(&mut bananas).enq()?;
    buff_apples.read(&mut apples).enq()?; 
    
    //print data on screen    
    print_vector(&bananas, GLOBAL_WORK_SIZE);
    print_vector2d(&apples, GLOBAL_WORK_SIZE/LOCAL_WORK_SIZE, LOCAL_WORK_SIZE);

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