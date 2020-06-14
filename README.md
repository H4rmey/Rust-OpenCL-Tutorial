# Rust-OpenCL-Tutorial

# Introduction:
Welcome to this OpenCL guide for the programming language Rust. My name is Harm Meyer I’m a third year technical informatics student at Avans Hogeschool in 's Hertogenbosch. I want to show you how you can use OpenCL in rust. This guide was written because of the lack of guides online that help with programming. Because of that this guide will mostly be a programming guide, not a guide that goes in depth on the inner workings of OpenCL.

In this git page you will find 5 pieces of example code. We are going to use these to learn about OpenCL, please open the code corresponding to the tutorial you are currently at. please note that these tutorials will be written in Rust, but can also be used in any other programming language that supports OpenCL. This is because most implementations of OpenCL follow the same steps. Furthermore, for this tutorial basic knowledge of the rust programming language is expected. The niveau I expect you to be on is where you know what cargo is, what a cargo.toml is and how basic rust syntax works. Sometimes I will explain things that might be new to you.



Finally it is expected that you know how OpenCL works. The only thing this tutorial will do is describe the code needed to use it. Some of the terms I expect you to know are: Kernel, Work sizes and global-, local-, and private memory. If you do not know about this I highly recommend the videos of David Gohara: [Youtube Playlist Link](https://www.youtube.com/watch?v=QA483lIvL-4&list=PLTfYiv7-a3l7mYEdjk35wfY-KQj5yVXO2)

----------------------------------------------------------------------------------------------------
# Tutorial 1 - Basics
## Part 1 - lets get started
Just like with everything we're going to start with the basics open "Tutorial1-Basics". In this file we will notice a couple of things. First being the dependencies, for this tutorial we are going to use "ocl", "ocl-core" and "ocl-extras". These dependencies can be found in the Cargo.toml of this project. These are the same in every tutorial unless specified otherwise. 

If we open our main.rs we can see at the top of the code that we've imported the external ocl crate and it's include objects. 

`extern crate ocl;
use ocl::{Result as OclResult, Platform, Device, Context, Queue, Buffer, Program, Kernel, EventList, SpatialDims};`

----------------------------------------------------------------------------------------------------
## part 2 - setup for all future programs
Underneath that the main_program() function is defined. Note how it returns an OclResult<()>. you'll need to do this if you want to know if there are any errors in your kernel code. all of our code will be ran in 

Now let's dive into the code step by step. First we get a list of platforms and we pick out the platform we want to use. In our case it's going to be the only one we have installed which is the OpenCL platform (tip: don't know which index you should pick? print each value of the returned platforms). 

	//Get all platforms
	//Pick one platform (OpenCL in our case)
	let platforms = Platform::list();

Next we select the device. You can see this as the processor that we're going to use. If you are unsure about what you should pick, loop through each index and select the one you need. Because i run this on a laptop with integrated graphics i simply use the first (and only) value my list of devices has :). 

	//Get all devices
	//Pick one device (The actual processor)      
	let devices = Device::list_all(platform);


Now we want to create the context and the program. The context is where all of your kernels are going to run. The program is you whole program object. This manages the calls to the host which will use the kernels within the context. 

    //create a context for the program (using the platform of course)
	let context = Context::builder().platform(*platform).build()?;

The src in the program is a string that is present in the beginning of the code. We'll go over it in the next chapter. 

	//create a program with a source, device and context
	let program = Program::builder().src(SRC).devices(device).build(&context)?;

Finally we create a Queue within our context using our device.

	//create a queue on the context and device
	let queue = Queue::new(&context, device, None)?; 

All of this was a setup se we can start the actual programming. Now that we've set this up we can move on to part 3.

----------------------------------------------------------------------------------------------------
## part 3 - Creating our kernel code
We've setup our basic environment but this won't actually do anything as of now. For this example we are going to make a vector. This vector will consist out of only ones. Of course you could put whatever you want in this vector, but I just want to insert ones.

	//create an array of 64 values that are all '1'
	//{1, 1, 1, 1, ... 1, 1}
	const MAX_SIZE : i32 = 64;
    	let mut bananas = vec![1; (MAX_SIZE) as usize]; 

Now we want to add 6 to each of the vector entries. Normally we would do something like this:

	for i in 0..MAX_SIZE
	{
		bananas[i] += 6;
	}

This is a sequential way to change the vector values. And it's this is a pretty simple/fast method for 64 values. But what if we want to change 640 or 6400 values. Now our simple for-loop is going to take a lot longer. To make sure our program runs as fast as it can we can use opencl for parrallel processing.

To do this we will first write a piece of kernel code. This is going to be the SRC variable at the top of our code. 

	static SRC: &'static str = r#"
	    __kernel void add(__global int *idBuffX)
	    {
		int globalIdX = get_global_id(0);
		
		idBuffX[globalIdX] += 6;
	    }
	"#;

First things first. Yes this is standard c code. the "__kernel" shows that this is a kernel function. We have not yet created a kernel but, we're starting with writing a piece of code that the kernel can actually use.

The "__global" shows that this variable exists in global memory. Next we expect a int pointer, this I s the buffer we're going to give to our kernel. Lastly we can see that we obtain the global id of the kernel thread and add six to each value of the buffer we passed in the arguments. 

To clarify: The idBuffX variable is an array. We want to select all the indexes of the array and add six to the values on those positions. Each thread of the kernel runs parallel to each other, this means they all execute this exact piece of code at the same time. But they all get a different value from the get_global_id() function. This way we change all the array values at the same time.

To further clarify: imagine running code on one pc that changes the array values. This pc can only run one piece of code at a time so we are forced to use a loop to change all the values of this array. This is not ideal, we want to change all array values at the same time because it's faster. To fix this we use multiple pc's and assign each one of these pc's an id. This id tells the pc which array value it needs to change. All the different pc's are connected to the same hard drive where the array data is stored (global memory). And finally we need a separate pc to manage all the pc's, a master pc that controls everything (the context). 

## part 4 - creating a buffer
Next we want to create a buffer for the kernel to use. This buffer will be used for the "__global int * idBufferX" argument of our __kernel function. This buffer needs to contain our vector, queue, datatype, and vector type. 

	let buff_bananas = Buffer::<i32>::builder()
		.queue(queue.clone())
		.len(bananas.len())
		.copy_host_slice(&bananas)
		.build()?;

## part 5 - creating the kernel
We are finally at the part where we can create our kernel. because we've setup a lot of parts beforehand we can now easily create a kernel. It will need a program, name, queue, work size and argument. 

	let kernel = Kernel::builder()
		.program(&program)
		.name("add")
		.queue(queue.clone())
		.global_work_size(MAX_SIZE)
		.arg(&buff_bananas)
		.build()?;

program: program object we've defined before. This has the Context, Source and Device build in. 

name: name of the function you want to use. The kernel will search the Program object for a function with the name "add". (Note: we can name the function whatever we want).

queue: copy of the queue object

global_work_size: the work size, in this case (and in most cases) the same the size of our vector. This function will create MAX_SIZE amount of objects that all run the same piece of code. This also decides how much id's the get_global_id() function will return. If we were to set this to 10, only the first 10 values of our vector would be changed. You could also see this as the amount times you want to loop through our vector.

arg: the first argument of our "add" function.

##part 6 - running and reading from the kernel
Next we want to run our kernel code and read the values from it. In Rust we need to run this code in an "unsafe" block. Because this code is not memory safe! for more information you could go to: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
    
To run the kernel we'll have to enqueue it:
 
	unsafe
	{
		kernel.cmd().enq();
	}

And finally we'll read the buffer we've created earlier and print out its values: 

	//read the values from the kernel  
	buff_bananas.read(&mut bananas).enq()?; 

	//print data on screen
	for banana in bananas
	{
		print!("{}\t", banana);
	}
	println!();

OUTPUT: 
    `{7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7}`

End of tutorial 1, next one will be shorter :)


# Tutorial 2 - Introduction
This tutorial we are going to make an adder. This adder will add the values of two arrays so that:

`{1,2,3,4} + {1,2,3,4} = {2,4,6,8}`

This time I will also rush through most concepts without adding new information. This tutorial is a lot less explaining and a lot more working.

Let's start the coding!

Unlike the first tutorial we are going to jump right into it. So we start up by getting our platform, device, context, program and queue ready!

	let platforms = Platform::list();
	let platform = &platforms[0];


	let devices = Device::list_all(platform);
	let device = devices?[0];

	let context = Context::builder().platform(*platform).build()?;

	let program = Program::builder().src(SRC).devices(device).build(&context)?;
	let queue = Queue::new(&context, device, None)?;

Next we want to create the source for our code:

	static SRC: &'static str = r#"
	    __kernel void add(__global int *buff_x, __global int *buff_y)
	    {
		int global_id_x = get_global_id(0);

		buff_x[global_id_x] += buff_y[global_id_x];
	    }
	"#;

This time we have two arguments because we have two arrays that need to be added to each other. We want to add the values of buff_x and buff_y. So we get the global_id of each piece of code and use that id to index on the array. Remember: this all happens in parallel!

Now that we've set this up we can create two buffers. These two buffers have the exact same values. They each count up from 0 to 8 so the look like: {0,1,2,3,4,5,6,7}.

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

To actually use all these object we are going to make the kernel object.

	`let kernel = Kernel::builder()
		.program(&program)
		.name("add")
		.queue(queue.clone())
		.global_work_size(MAX_SIZE)
		.arg(&buff_bananas)
		.arg(&buff_apples)
		.build()?;`

note: I use the arg method two times for the first and second argument needed for the add() function in the source of the kernel. And finally we simply enqueue the kernel and read the values from the buffers.

    //enqueue the kernel
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

That's it, we've created an OpenCL program that adds the values of two arrays together. We can run this code by typing "Cargo Run" and it will show the following array output: 

`{0,2,4,6,8,10,12,14}`

This code can also be used for multiplication, subtraction and dividing. 

# Tutorial 3 - Dimensions
Within our kernel we can work with up to 3 dimensions. These dimensions could be seen as a for-loop within a for loop. Normally in almost any programming language, if we have an array that has two dimensions we would do something like this:

	for x in 0..x_range
	{
		for y in 0..y_range
		{
			array[x][y] = 1;
		}
	}

In OpenCL it works a bit differently. For this example we will go up to a 2d array to keep it somewhat simple. But do note that this could also work with 3d arrays. To index the value we want or to store a value in a 2d array we can also use a 1d array that has the size of x_range * y_range. (After all a 2d array is nothing but pointers that point to other arrays that have more pointers) To 

## part 1 - storing 2d array in a 1d array
To create a 2d array that countains all it's values in a 1d square array i created a simple function.

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

		if (doDebug)
		{
			//function that loops through each value and prints out the value also self-written :)
		    print_vector2d(&temp, MAX_SIZE, MAX_SIZE);
		}

		temp
	}

When we enter a MAX_SIZE of 8 this function will return a Vec<i32> that is filled like this:

	`-----------------------------------------------------------------
	|0|     |1|     |2|     |3|     |4|     |5|     |6|     |7|
	|8|     |9|     |10|    |11|    |12|    |13|    |14|    |15|
	|16|    |17|    |18|    |19|    |20|    |21|    |22|    |23|
	|24|    |25|    |26|    |27|    |28|    |29|    |30|    |31|
	|32|    |33|    |34|    |35|    |36|    |37|    |38|    |39|
	|40|    |41|    |42|    |43|    |44|    |45|    |46|    |47|
	|48|    |49|    |50|    |51|    |52|    |53|    |54|    |55|
	|56|    |57|    |58|    |59|    |60|    |61|    |62|    |63|
	-----------------------------------------------------------------`

If we want to index a value at x=5 and y=6 we'll have to use vector_name[size_of_vector * x + y]. This will return value 53. From this point onward I will refer to the array as a vector when we are working with Rust directly and as a array when working with kernel components.

## part 2 - Using the kernel
So to start of I’m going to assume you've already completed tutorial 2 and know how to setup a basic kernel. Now we're going to edit his code a bit (you could also write it from scratch but honestly just copy and paste...). 

Just like in tutorial 2 we're going to add 2 vectors together. But we don't want to do that using for-loops we are going to use OpenCL. So we're going to make 2 vectors using our new create_countup_vector2d() function.

Note: the size of this vector is now MAX_SIZE * MAX_SIZE.

	let mut bananas = create_countup_vector2d(MAX_SIZE, true);
    let buff_bananas = Buffer::<i32>::builder()
        .queue(queue.clone())
        .len(bananas.len())
        .copy_host_slice(&bananas)
        .build()?;

    let mut apples = create_countup_vector2d(MAX_SIZE, true);
    let buff_apples = Buffer::<i32>::builder()
        .queue(queue.clone())
        .len(apples.len())
        .copy_host_slice(&apples)
        .build()?;

Next we're going to create our kernel weirdly named salad, because I’m hungry. Again note that our global_work_size is MAX_SIZE * MAX_SIZE. In rust this is done by the SpatialDims object of the ocl library. Now our kernel knows it has two working dimensions.

    let salad = Kernel::builder()
        .program(&program)
        .name("add")
        .queue(queue.clone())
        .global_work_size(SpatialDims::Two(MAX_SIZE, MAX_SIZE))
        .arg(&buff_bananas)
        .arg(&buff_apples)
        .build()?;

Now that we've put our salad together with bananas and apples. Bring our attention to the source of the kernel.


## Part 3 - Kernel code
The kernel code in this case is not as impressive as you might have assumed. First we obtain the global id's of each thread. To get the id of higher dimensions you simple input a value from 0 to 3. To select our second dimension for example we use get_global_id(1).

	__kernel void add(__global int *buff_x, __global int *buff_y)
    {
        int global_id_x = get_global_id(0);
        int global_id_y = get_global_id(1);
        
        int new_id = get_global_size(0) * global_id_x + global_id_y;

        buff_x[new_id] += buff_y[new_id];
    }

After we've gotten our id's we simply use the same formula to index the array value and add them together.

Finally we read the values from our apples and bananas buffer and read the result. All values in the bananas buffer should now look something like this:

	-----------------------------------------------------------------
	|0|     |2|     |4|     |6|     |8|     |10|    |12|    |14|
	|16|    |18|    |20|    |22|    |24|    |26|    |28|    |30|
	|32|    |34|    |36|    |38|    |40|    |42|    |44|    |46|
	|48|    |50|    |52|    |54|    |56|    |58|    |60|    |62|
	|64|    |66|    |68|    |70|    |72|    |74|    |76|    |78|
	|80|    |82|    |84|    |86|    |88|    |90|    |92|    |94|
	|96|    |98|    |100|   |102|   |104|   |106|   |108|   |110|
	|112|   |114|   |116|   |118|   |120|   |122|   |124|   |126|
	-----------------------------------------------------------------

That was all for tutorial 3!

## Tutorial 4 - private- local- and global memory
So in this example we're going to calculate the sum of all the values in a vector. To do this we're going to use a bit of local an private memory and show how you can use this.

## Part 1 - writing the kernel code
right of the bat we're going to start with the kernel code. This is because this tutorial focusses more on analysis of the working of the kernel than it does on the working of the Rust code.

	__kernel void add(__global int *in, __global int *out, __local int *temp)
    {        
        int sum;
        uint local_id;

        local_id = get_local_id(0) * 2;

        temp[local_id] = in[local_id] + in[local_id+1]; 

        barrier(CLK_LOCAL_MEM_FENCE);

        if(get_local_id(0) == 0) 
        {
            sum = 0;
            for(int i=0; i < get_local_size(0); i++) 
            {
                sum += temp[i];
            }
            out[0] = sum;
        }
    }

In the code above we expect an int array in, an int array out and a __local int *temp. The last argument is shared memory between the threads. Every instance of this code can use this variable and edit it, just like with __global values. The main difference is that local variables cannot be read outside of the kernel (what happens in the kernel stays in the kernel). 

But why use this when we can just use __global? Because this __local is much faster for with memory transfer because it's physically much closer to the processing unit then the __global memory. 

First we're going to initialize a sum and global_id variable. Next each instance of code will add 2 numbers together. So the instance with id 0 will add the values 0 and 1 and so on:

	0 => 0 + 1
	1 => 2 + 3
	2 => 4 + 5
		.
		.
		.
	n => 2n + 2n+1

All these values get stored in the temp value. After that we loop over each index of the temp value and add those values. (this might not be an ideal sollution but it's more the idea of how __local works) 

## Part 2 - Barriers
Sometimes you need to syncronise all the threads running. We can do this by using a barrier. In the kernel code above we can see the line: 

   barrier(CLK_LOCAL_MEM_FENCE);`

This barrier waits for all threads to reach the same position and flushed the local memory. Next only the thread with id 0 adds all the values of temp together. Please note that all threads run through this code but only thread 0 will actually execute it. Finally we write the sum of the values to our output.

That was all for tutorial 4

# Tutorial 5 - Local and Global workspaces
Up until now we've always used the get_global_id() function to get the index of an array value. Now I would like to introduce you to the world of get_local_id(). We can set a local workspace to split the global workspace. This means we get a second index which is very helpful for splitting work. 

Normally we would have a bunch of global_id's like this when we set a work size of 24

 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 

Now if we set a local worksize of 4 we get a second index we can use. Now our indexing wil look like this:

 Global ID | #0 | #1 | #2 | #3 | #4 | #5 | #6 | #7 | #8 | #9 | #10 | #11 | #12 | #13 | #14 | #15 | #16 | #17 | #18 | #19 | #20 | #21 | #22 | #23
 Local ID | 0 | 1 | 2 | 3 | 0 | 1 | 2 | 3 | 0 | 1 | 2 | 3 | 0 | 1 | 2 | 3 | 0 | 1 | 2 | 3 | 0 | 1 | 2 | 3 

But the local workspace is more than just a second index we can use. It splits the global workspace in groups. Our workspace of 24 is split in groups of 24/4=6. So we've split our 24 threads into 6. These 6 threads of size 4, each have their own local memory.

	__kernel void add(__global int *bananas, __global int *apples)
	{  
        bananas[get_local_id(0)] = get_global_id(0);
        apples[get_global_id(0)] = get_local_id(0);
	}

In the example code you can see i have added:
        
	.local_work_size(LOCAL_WORK_SIZE) //LOCAL_WORK_SIZE = 4 

This will add a local worksize of 4.

Now if we run this code and check the values of our buffers we can see: 

The value of buff_bananas is:
 20  21  22  23  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0  0 

This makes sense because the kernel code wants us to index of the local_id (bananas[get_local_id(0)]) this goes from 0 to 3 so only the first 4 values are filled in. Next we set it to the get_global_id(0). The last threads to finish were those of id's 20, 21 22 and 23. 

The value of buff_apples is:
|0|     |1|     |2|     |3|
|0|     |1|     |2|     |3|
|0|     |1|     |2|     |3|
|0|     |1|     |2|     |3|
|0|     |1|     |2|     |3|
|0|     |1|     |2|     |3|

This also makes sense because we index on the global_id, so all the values that are of the array are filled! The value loops through the values 0 to 3 because we have a local_size of 4. 

That was all for tutorial 5
