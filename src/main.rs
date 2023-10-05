// Uncomment these following global attributes to silence most warnings of "low" interest:

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unreachable_code)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_variables)]

extern crate nalgebra_glm as glm;
use std::{ mem, ptr, os::raw::c_void };
use std::thread;
use std::sync::{Mutex, Arc, RwLock};

mod shader;
mod util;
mod mesh;
mod scene_graph;
mod toolbox;

use glm::{Vec3, vec4, vec3, Mat4};
use glutin::event::{Event, WindowEvent, DeviceEvent, KeyboardInput, ElementState::{Pressed, Released}, VirtualKeyCode::{self, *}};
use glutin::event_loop::ControlFlow;
use mesh::Helicopter;
use scene_graph::SceneNode;
use toolbox::Heading;

// initial window size
const INITIAL_SCREEN_W: u32 = 800;
const INITIAL_SCREEN_H: u32 = 600;

// == // Helper functions to make interacting with OpenGL a little bit prettier. You *WILL* need these! // == //

// Get the size of an arbitrary array of numbers measured in bytes
// Example usage:  pointer_to_array(my_array)
fn byte_size_of_array<T>(val: &[T]) -> isize {
    std::mem::size_of_val(&val[..]) as isize
}

// Get the OpenGL-compatible pointer to an arbitrary array of numbers
// Example usage:  pointer_to_array(my_array)
fn pointer_to_array<T>(val: &[T]) -> *const c_void {
    &val[0] as *const T as *const c_void
}

// Get the size of the given type in bytes
// Example usage:  size_of::<u64>()
fn size_of<T>() -> i32 {
    mem::size_of::<T>() as i32
}

// Get an offset in bytes for n units of type T, represented as a relative pointer
// Example usage:  offset::<u64>(4)
fn offset<T>(n: u32) -> *const c_void {
    (n * mem::size_of::<T>() as u32) as *const T as *const c_void
}

// Get a null pointer (equivalent to an offset of 0)
// ptr::null()


// == // Generate your VAO here
unsafe fn create_vao(vertices: &Vec<f32>, indices: &Vec<u32>, color: &Vec<f32>, normal: &Vec<f32>) -> u32 {

    // This should:
    // * Generate a VAO (vertex array object) and bind it
    let mut vao_id:u32 =0;
    gl::GenVertexArrays(1, &mut vao_id);
    gl::BindVertexArray(vao_id);

    // * Generate a VBO (Vertex buffer object) and bind it
    let mut vbo_id:u32=0;
    gl::GenBuffers(1, &mut vbo_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, vbo_id);
    // * Fill it with data
    gl::BufferData(
        gl::ARRAY_BUFFER, 
        byte_size_of_array(&vertices) as isize, // vert?
        vertices.as_ptr().cast(),     
        gl::STATIC_DRAW
    );

    // * Configure a VAP for the data and enable it
    gl::VertexAttribPointer(
        0, // 0 or add u32?
        3,
        gl::FLOAT,  //should match data type from buffer data
        gl::FALSE,
        0,
        std::ptr::null()
    );
    gl::EnableVertexAttribArray(0);

    // * Generate a IBO and bind it
    let mut ibo_id:u32=0;
    gl::GenBuffers(1, &mut ibo_id);
    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ibo_id);
    //* fill IBO with data
    gl::BufferData(
        gl::ELEMENT_ARRAY_BUFFER,
        byte_size_of_array(&indices),
        indices.as_ptr().cast(),
        gl::STATIC_DRAW
    );

    //Color Generate, bind and fill with data, VBO for color
    let mut color_id:u32=0;
    gl::GenBuffers(1, &mut color_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, color_id);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&color),
        color.as_ptr().cast(),
        gl::STATIC_DRAW
    );

    gl::VertexAttribPointer(
        1,
        4,
        gl::FLOAT,
        gl::FALSE, // setter normalised, skal kanskje være true
        0,
        std::ptr::null()
    );

    gl::EnableVertexAttribArray(1);


    // normals:
    let mut normals_id:u32=0;
    gl::GenBuffers(1, &mut normals_id);
    gl::BindBuffer(gl::ARRAY_BUFFER, normals_id);
    gl::BufferData(
        gl::ARRAY_BUFFER,
        byte_size_of_array(&normal),
        normal.as_ptr().cast(),
        gl::STATIC_DRAW
    );
    //println!("normals_id\t: {:?}",&normal);

    gl::VertexAttribPointer(
        2,
        3,
        gl::FLOAT,
        gl::FALSE, // setter normalised til true
        0,
        std::ptr::null()
    );


    gl::EnableVertexAttribArray(2);







    // * Return the ID of the VAO
    return vao_id;
}


// draw scene ---------------------------------------------------------------
unsafe fn draw_scene(

    node: &scene_graph::SceneNode,
    view_projection_matrix: &glm::Mat4,
    transformation_so_far: &glm::Mat4) {
    // Perform any logic needed before drawing the node
    //the translations and rotations are not yet foolproof, might have done it wrong, the angle is probably way off
    
    let mut transformation: glm::Mat4 = glm::identity();
    //translate
    //node.reference_point = transformation_so_far.dot(node.reference_point);

    transformation =  glm::translation(&(-node.reference_point)) * transformation;
    //rotates if there is a defined matrix (hopefully)
    transformation = glm::rotation(node.rotation.z,&glm::vec3(0.0, 0.0, 1.0)) * transformation;
    transformation = glm::rotation(node.rotation.y,&glm::vec3(0.0, 1.0, 0.0)) * transformation;
    transformation = glm::rotation(node.rotation.x,&glm::vec3(1.0, 0.0, 0.0)) * transformation; 
    //scale?
    
    //translate back 
    transformation =  glm::translation(&(node.reference_point)) * transformation;

    transformation =  glm::translation(&(node.position)) * transformation;

    //let mut holder = transformation * vec4 (node.reference_point.x, node.reference_point.y,node.reference_point.z,1.0);
    //transformation_so_far = vec3()


    //add to transformation so far:
    let model_matrix: glm::Mat4 = transformation_so_far * transformation;
    // let MVP: glm::Mat4 =  view_projection_matrix * model_matrix; //redundant
    //let _ = transformation_so_far * model_matrix;


    // Check if node is drawable, if so: set uniforms, bind VAO and draw VAO
    if node.index_count != -1{ // this might be 2 or three

        //uniforms:

        unsafe{
            // sending the matrix to vertex shader
            //gl::UseProgram(shader.program_id);
            gl::UniformMatrix4fv(10, 1,0, model_matrix.as_ptr());
            gl::UniformMatrix4fv(26, 1, 0, view_projection_matrix.as_ptr());
        }

        //bind and draw VAO
        gl::BindVertexArray(node.vao_id);
        gl::DrawElements(
        gl::TRIANGLES,
        node.index_count,
        gl::UNSIGNED_INT,
        std::ptr::null()
        );
    }
    // Recurse
    for &child in &node.children {
    draw_scene(&*child, view_projection_matrix, &model_matrix);
    }
    }


// ----------------------------------------------------------------------------------------

fn main() {
    
    // Set up the necessary objects to deal with windows and event handling
    let el = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Gloom-rs")
        .with_resizable(true)
        .with_inner_size(glutin::dpi::LogicalSize::new(INITIAL_SCREEN_W, INITIAL_SCREEN_H));
    let cb = glutin::ContextBuilder::new()
        .with_vsync(true);
    let windowed_context = cb.build_windowed(wb, &el).unwrap();
    // Uncomment these if you want to use the mouse for controls, but want it to be confined to the screen and/or invisible.
    // windowed_context.window().set_cursor_grab(true).expect("BLENDfailed to grab cursor");
    // windowed_context.window().set_cursor_visible(false);

    // Set up a shared vector for keeping track of currently pressed keys
    let arc_pressed_keys = Arc::new(Mutex::new(Vec::<VirtualKeyCode>::with_capacity(10)));
    // Make a reference of this vector to send to the render thread
    let pressed_keys = Arc::clone(&arc_pressed_keys);

    // Set up shared tuple for tracking mouse movement between frames
    let arc_mouse_delta = Arc::new(Mutex::new((0f32, 0f32)));
    // Make a reference of this tuple to send to the render thread
    let mouse_delta = Arc::clone(&arc_mouse_delta);

    // Set up shared tuple for tracking changes to the window size
    let arc_window_size = Arc::new(Mutex::new((INITIAL_SCREEN_W, INITIAL_SCREEN_H, false)));
    // Make a reference of this tuple to send to the render thread
    let window_size = Arc::clone(&arc_window_size);

    // Spawn a separate thread for rendering, so event handling doesn't block rendering
    let render_thread = thread::spawn(move || {
        // Acquire the OpenGL Context and load the function pointers.
        // This has to be done inside of the rendering thread, because
        // an active OpenGL context cannot safely traverse a thread boundary
        let context = unsafe {
            let c = windowed_context.make_current().unwrap();
            gl::load_with(|symbol| c.get_proc_address(symbol) as *const _);
            c
        };

        let mut window_aspect_ratio = INITIAL_SCREEN_W as f32 / INITIAL_SCREEN_H as f32;

        // Set up openGL
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);
            //gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::MULTISAMPLE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            gl::DebugMessageCallback(Some(util::debug_callback), ptr::null());

            // Print some diagnostics
            println!("{}: {}", util::get_gl_string(gl::VENDOR), util::get_gl_string(gl::RENDERER));
            println!("OpenGL\t: {}", util::get_gl_string(gl::VERSION));
            println!("GLSL\t: {}", util::get_gl_string(gl::SHADING_LANGUAGE_VERSION));
        }

        // == // Set up your VAO around here

        let num_tre: i32= 5;
        let count_tri: i32 = num_tre*3;
            

        
        let vertices: Vec<f32> = vec!
        /* 
        [0.6, -0.8, -1.2,
        0.0, 0.4, 0.0,
        -0.8, -0.2, 1.2];
        */
        [
         -0.6, 0.0, 0.5,
         0.0, 0.0, 0.2,
         -0.3, 0.6, 0.5,

         -0.4, 0.0, -0.1,
         0.2, 0.0, -0.5,
         -0.1, 0.6, -0.5,

         -0.5, -0.1, -0.9,
         0.1, -0.1, -0.9,
         -0.2, 0.5, -0.6,

         -10.6,-5.0, -10.0,
         20.4, -4.0, -10.0,
         0.5, 15.6, -10.0,

         -10.6,-5.0, 10.0,
         20.4, -4.0, 10.0,
         0.5, 21.6, 10.0
            
        ];
        
        /*
        [-0.6, 0.4, 0.0,
         -0.4, 0.4, 0.0,
         -0.5, 0.6, 0.0,

         -0.3, 0.4, 0.0,
         -0.1, 0.4, 0.0,
         -0.2, 0.7, 0.0,

         -0.7, 0.1, 0.0,
         -0.4, 0.1, 0.0,
         -0.4, 0.3, 0.0,

         -0.3, 0.1, 0.0,
         -0.0, 0.2, 0.0,
         -0.2, 0.3, 0.0,

         -0.3, -0.2, 0.0,
         -0.1, -0.2, 0.0,
         -0.2, 0.0, 0.0

        ];
        */
        
        let indices: Vec<u32> = vec!
        [0, 1, 2,
        3, 4, 5,
        6, 7, 8,
        9, 10,11,
        12, 13, 14];

        let color: Vec<f32> = vec![
            0.0, 0.0, 1.0, 0.4,
            0.0, 0.0, 1.0, 0.4,
            0.0, 0.0, 1.0, 0.4,

            1.0, 0.0, 0.0, 0.6,
            1.0, 0.0, 0.0, 0.6,
            1.0, 0.0, 0.0, 0.6,

            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,
            0.0, 1.0, 0.0, 0.5,

            1.0, 0.4, 0.3, 1.0,
            0.1, 1.0, 1.0, 1.0,
            0.1, 0.0, 1.0, 1.0,

            0.5, 1.0, 0.0, 1.0,
            0.5, 0.4, 1.0, 1.0,
            1.0, 0.0, 1.0, 1.0

            // 0.9, 0.0, 0.8, 1.0,
            // 0.0, 0.8, 0.8, 1.0,
            // 0.9, 0.8, 0.0, 1.0
            // //,

            // 0.4, 0.0, 0.1, 1.0,
            // 0.9, 0.9, 0.0, 1.0,
            // 0.0, 0.5, 0.6, 1.0,

            // 1.0, 0.0, 0.0, 1.0,
            // 0.0, 1.0, 0.0, 1.0,
            // 0.0, 0.0, 1.0, 1.0
        ];
        
        let normal: Vec<f32> = vec![

        ];
        // let my_vao = unsafe { 
        //     create_vao(&vertices, &indices, &color, &normal)
        // };


        // == // Set up your shaders here
        let shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./shaders/simple.frag")
                .attach_file("./shaders/simple.vert")
                .link()
        };
        // load terrain
        let terrain= mesh::Terrain::load("./resources/lunarsurface.obj");
        

        let my_terrain = unsafe{ 
            create_vao(
                &terrain.vertices,
                &terrain.indices,
                &terrain.colors,
                &terrain.normals) 
        };

        // load helicopters



        let helicopter = mesh::Helicopter::load("./resources/helicopter.obj");
        let heli_body = helicopter.body;
        let heli_door = helicopter.door;
        let heli_m_rotor = helicopter.main_rotor;
        let heli_t_rotor = helicopter.tail_rotor;


        let my_heli_body = unsafe{
            create_vao(
                &heli_body.vertices, 
                &heli_body.indices, 
                &heli_body.colors,
                &heli_body.normals)
        };

        let my_heli_door = unsafe{
            create_vao(
                &heli_door.vertices, 
                &heli_door.indices, 
                &heli_door.colors,
                &heli_door.normals)
        };

        let my_heli_m_rotor = unsafe{
            create_vao(
                &heli_m_rotor.vertices, 
                &heli_m_rotor.indices, 
                &heli_m_rotor.colors,
                &heli_m_rotor.normals)
        };
        let my_heli_t_rotor = unsafe{
            create_vao(
                &heli_t_rotor.vertices, 
                &heli_t_rotor.indices, 
                &heli_t_rotor.colors,
                &heli_t_rotor.normals)
        };


        // // !!!!!!!!!!!!!!!  AFFINE MATRIX TRANSFORMATIONS !!!!!!!!
            //denne skal inn i shaderen, usikker på metoden fortsatt.
        // mat4x4 AffineTM = {{1.0 0.0, 0.0, 0.0},{0.0, 1.0, 0.0, 0.0},{0.0, 0.0, 1.0, 0.0},{0.0, 0.0, 0.0, 1.0}};



        // //choosing which transformation matrix to send to the Shader:

        // let AffineTM = M;

        // Basic usage of shader helper:
        // The example code below creates a 'shader' object.
        // It which contains the field `.program_id` and the method `.activate()`.
        // The `.` in the path is relative to `Cargo.toml`.
        // This snippet is not enough to do the exercise, and will need to be modified (outside
        // of just using the correct path), but it only needs to be called once

///////
        //unsafe{
        //gl::BindVertexArray(my_vao) 
        //}
///////


        /*
        let simple_shader = unsafe {
            shader::ShaderBuilder::new()
                .attach_file("./path/to/simple/shader.file")
                .link()
        };
        */


        // Used to demonstrate keyboard handling for exercise 2.
        //motion variables:
        let mut transx_val: f32 = 0.0;
        let mut transy_val: f32 = 0.0;
        let mut transz_val: f32 = 0.0;
        let mut rotx_val: f32 =0.0;
        let mut roty_val: f32 =0.0;
        
        let trans_speed: f32 = 20.0;
        let rot_speed: f32 = 1.0;

        let mut xVec: glm::Vec4 = glm::vec4(1.0, 0.0, 0.0, 0.0);
        let mut yVec: glm::Vec4 = glm::vec4(0.0, 1.0, 0.0, 0.0);
        let mut zVec: glm::Vec4 = glm::vec4(0.0, 0.0, 1.0, 0.0);





        // The main rendering loop
        let first_frame_time = std::time::Instant::now();
        let mut previous_frame_time = first_frame_time;
        loop {
            // Compute time passed since the previous frame and since the start of the program
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(first_frame_time).as_secs_f32();
            let delta_time = now.duration_since(previous_frame_time).as_secs_f32();
            previous_frame_time = now;



            // Handle resize events
            if let Ok(mut new_size) = window_size.lock() {
                if new_size.2 {
                    // 0.4, 0.0, 0.1, 1.0,
                    // 0.9, 0.9, 0.0, 1.0,
                    // 0.0, 0.5, 0.6, 1.0,
        
                    // 1.0, 0.0, 0.0, 1.0,
                    // 0.0, 1.0, 0.0, 1.0,
                    // 0.0, 0.0, 1.0, 1.0
                    println!("Window was resized to {}x{}", new_size.0, new_size.1);
                    unsafe { gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32); }
                }
            }

            // // Handle keyboard input

            //key press handlers:
            // Handle keyboard input
            if let Ok(keys) = pressed_keys.lock() {
                for key in keys.iter() {
                    match key {
                        // The `VirtualKeyCode` enum is defined here:
                        //    https://docs.rs/winit/0.25.0/winit/event/enum.VirtualKeyCode.html

                        VirtualKeyCode::A => {
                            transx_val += delta_time*trans_speed;
                        }
                        VirtualKeyCode::D => {
                            transx_val -= delta_time*trans_speed;
                        }

                        VirtualKeyCode::LShift => {
                            transy_val += delta_time*trans_speed;
                        }
                        VirtualKeyCode::Space => {
                            transy_val -= delta_time*trans_speed;
                        }

                        VirtualKeyCode::W => {
                            transz_val += delta_time*trans_speed;
                        }
                        VirtualKeyCode::S => {
                            transz_val -= delta_time*trans_speed;
                        }


                        VirtualKeyCode::Down => {
                            rotx_val += delta_time*rot_speed;
                        }
                        VirtualKeyCode::Up => {
                            rotx_val -= delta_time*rot_speed;
                        }

                        VirtualKeyCode::Right => {
                            roty_val += delta_time*rot_speed;
                        }
                        VirtualKeyCode::Left => {
                            roty_val -= delta_time*rot_speed;
                        }



                    


                        // default handler:
                        _ => { }
                    }
                }
            }



            // Handle mouse movement. delta contains the x and y movement of the mouse since last frame in pixels
            if let Ok(mut delta) = mouse_delta.lock() {

                // == // Optionally access the accumulated mouse movement between
                // == // frames here with `delta.0` and `delta.1`

                *delta = (0.0, 0.0); // reset when done
            }



                        //



                        //let s= elapsed.sin();
                        // let c = elapsed.cos();
            
                        // //constants:
            
                        // //vectors
                        //let xVec: glm::Vec3 = glm::vec3(s, 0.0, 0.0);
                        // let yVec: glm::Vec3 = glm::vec3(0.0, 1.0, 0.0);
                        // let zVec: glm::Vec3 = glm::vec3(0.0, 0.0, 1.0);
                        // let iVec: glm::Vec3 = glm::vec3(1.0, 1.0, 1.0);
                        // // constants
                        // let angle: f32 = 1.0;
            
                        // //Matrices:
            
                        // //rotation:
                        // let rotx: glm::Mat4 = glm::rotation(angle, &xVec);
                        // let roty: glm::Mat4 = glm::rotation(angle, &yVec);
                        // let rotz: glm::Mat4 = glm::rotation(angle, &zVec);
            
                        // //translation:
                        // let transx: glm::Mat4 = glm::translation(&xVec);
                        // let transy: glm::Mat4 = glm::translation(&yVec);
                        // let transz: glm::Mat4 = glm::translation(&zVec);
            
                        //identity
                        // let identity: glm::Mat4 = glm::identity();


                        //
//start of coding task 4 ø2

            // == // Please compute camera transforms here (exercise 2 & 3)
            //dealarations
            //projection deaclaration
            let projection: glm::Mat4 = glm::perspective(window_aspect_ratio, 90.0, 1.0, 1000.0); // increased far to 1000



            // // movement axis:
            // let xVec: glm::Vec3 = glm::vec3(x1, 0.0, 0.0);
            // let yVec: glm::Vec3 = glm::vec3(0.0, y1, 0.0);
            // let zVec: glm::Vec3 = glm::vec3(0.0, 0.0, z1);

            //this is for attempting to fix the translation axies.
            // xVec = rot_y * rot_x * xVec;
            // yVec = rot_y * rot_x * yVec;

            // let mut xVec3: glm::Vec3 = xVec;
            // let yVec3: glm::Vec3 = yVec;


            // glm::translation(&xVec3 * transx_val)






        //  // matrix multiplications goes here:
        let rot_y:glm::Mat4= glm::rotation(roty_val,&glm::vec3(0.0, 1.0, 0.0));
        let rot_x:glm::Mat4= glm::rotation(rotx_val,&glm::vec3(1.0, 0.0, 0.0));


            let mut trans: glm::Mat4= glm::identity(); //final computed matrix
            trans = glm::translation(&glm::vec3(0.0, 0.0, -2.0)) * trans;
            trans = glm::translation(&glm::vec3(transx_val, transy_val, transz_val)) * trans;
            trans = rot_y * trans;
            trans = rot_x * trans;

            //this must always be last!
            trans = projection *trans;


            // unsafe{
            //     // sending the matrix to vertex shader
            //     gl::UseProgram(shader.program_id);
            //     gl::UniformMatrix4fv(10, 1,0, trans.as_ptr());
            // }

            let rps_tail = 2.0;
            let rot_scale_tail = rps_tail *2.0*3.14;
            let mut rotation_tail = rot_scale_tail* elapsed;

            let rps_main = 1.0;
            let rot_scale_main = rps_main *2.0*3.14;
            let mut rotation_main = elapsed* rot_scale_main;

            unsafe {
                // Clear the color and depth buffers
                gl::ClearColor(0.035, 0.046, 0.078, 1.0); // night sky, full opacity
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                shader.activate();
                // //     terrain.index_count,
                // //     gl::UNSIGNED_INT,
                // //     std::ptr::null()

                // // );

                // gl::BindVertexArray(my_heli_body);
                // // gl::DrawElements(
                // //     gl::TRIANGLES,
                // //     heli_body.index_count,
                // //     gl::UNSIGNED_INT,
                // //     std::ptr::null()

                // // );

                // gl::BindVertexArray(my_heli_door);
                // // gl::DrawElements(
                // //     gl::TRIANGLES,
                // //     heli_door.index_count,
                // //     gl::UNSIGNED_INT,
                // //     std::ptr::null()

                // // );

                // gl::BindVertexArray(my_heli_m_rotor);
                // // gl::DrawElements(
                // //     gl::TRIANGLES,
                // //     heli_m_rotor.index_count,
                // //     gl::UNSIGNED_INT,
                // //     std::ptr::null()

                // // );
                // gl::BindVertexArray(my_heli_t_rotor);
                // // gl::DrawElements(
                //     // gl::TRIANGLES,
                //     // heli_t_rotor.index_count,
                //     // gl::UNSIGNED_INT,
                //     // std::ptr::null()

                // // );

            
            
            }
            //load heading
            //        let terrain= mesh::Terrain::load("./resources/lunarsurface.obj");
            //let heading = toolbox::simple_heading_animation::load("./src/toolbox.rs");
            let offset = 1.1;

            // let mut heading = toolbox::simple_heading_animation(elapsed+offset); //this should also be in the loop



            //let world_pos: Vec3= vec3(trans[(3, 0)], trans[(3,1)], trans[(3,2)]);
            let mut root_node = SceneNode::new();
            let mut scene_node = SceneNode::from_vao(my_terrain, terrain.index_count);
            root_node.add_child(&scene_node);
            //root_node.reference_point = -world_pos;

            //etter dette skal alt forhåpentlgvis funbgere via loopen::

            // let mut heli_bod_node = SceneNode::from_vao(my_heli_body, heli_body.index_count);
            // scene_node.add_child(&heli_bod_node);
            // heli_bod_node.reference_point = vec3(heading.x, 0.0, heading.z);
            // heli_bod_node.rotation = vec3(heading.roll, heading.yaw, heading.pitch);
            


            // let mut heli_door_node = SceneNode::from_vao(my_heli_door, heli_door.index_count);
            // heli_bod_node.add_child(&heli_door_node);
            // heli_door_node.reference_point = vec3(0.0, 0.0, 0.0);
            // //heli_door_node.rotation= glm::vec3(0.0, rotation_tail, 0.0);

            // let mut heli_m_rot_node = SceneNode::from_vao(my_heli_m_rotor, heli_m_rotor.index_count);
            // heli_bod_node.add_child(&heli_m_rot_node);
            // heli_m_rot_node.reference_point= glm::vec3(0.0, 2.3, 0.0);
            // heli_m_rot_node.rotation= glm::vec3(0.0, rotation_main, 0.0);

            // let mut heli_t_rot_node = SceneNode::from_vao(my_heli_t_rotor, heli_t_rotor.index_count);
            // heli_bod_node.add_child(&heli_t_rot_node);
            // heli_t_rot_node.reference_point= glm::vec3(0.35, 2.3, 10.4);
            // heli_t_rot_node.rotation= glm::vec3(rotation_tail, 0.0, 0.0);
            
            // // det over denne skal fungere fra loopen

            // heli_bod_node.print();
            // heli_door_node.print();
            // heli_m_rot_node.print();
            // heli_t_rot_node.print();
            ///////
            let offset = 3.2;
            let mut nodes: Vec<scene_graph::Node> = Vec::new();
            //nodes[0] = scene_node;

            for i in 0..5{
                let mut heading = toolbox::simple_heading_animation(elapsed + offset * &(i as f32));
                //heli body
                nodes.push(SceneNode::from_vao(my_heli_body, heli_body.index_count)); 
                scene_node.add_child(&nodes[i]);
                nodes[i].position = vec3(heading.x, 20.0, heading.z);
                
                //nodes[i].rotation = vec3(heading.roll, heading.yaw, heading.pitch);
                nodes[i].rotation = vec3(heading.pitch, heading.yaw, heading.roll);
                //Heli door
                nodes[i].add_child(&SceneNode::from_vao(my_heli_door, heli_door.index_count)); 
                nodes[i].get_child(0).reference_point = vec3(0.0, 0.0, 0.0);
                //heli main rotor
                nodes[i].add_child(&SceneNode::from_vao(my_heli_m_rotor, heli_m_rotor.index_count));
                nodes[i].get_child(1).reference_point= glm::vec3(0.0, 2.3, 0.0);
                nodes[i].get_child(1).rotation= glm::vec3(0.0, rotation_main, 0.0);
                //heli tail rotor
                nodes[i].add_child(&SceneNode::from_vao(my_heli_t_rotor, heli_t_rotor.index_count));
                nodes[i].get_child(2).reference_point= glm::vec3(0.35, 2.3, 10.4);
                nodes[i].get_child(2).rotation= glm::vec3(rotation_tail, 0.0, 0.0);





            }


            /////
            unsafe{

            draw_scene(&root_node,&trans, &glm::Mat4::identity());
            }


            // Display the new color buffer on the display
            context.swap_buffers().unwrap(); // we use "double buffering" to avoid artifacts
        }
    });


    // == //
    // == // From here on down there are only internals.
    // == //


    // Keep track of the health of the rendering thread
    let render_thread_healthy = Arc::new(RwLock::new(true));
    let render_thread_watchdog = Arc::clone(&render_thread_healthy);
    thread::spawn(move || {
        if !render_thread.join().is_ok() {
            if let Ok(mut health) = render_thread_watchdog.write() {
                println!("Render thread panicked!");
                *health = false;
            }
        }
    });

    // Start the event loop -- This is where window events are initially handled
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Terminate program if render thread panics
        if let Ok(health) = render_thread_healthy.read() {
            if *health == false {
                *control_flow = ControlFlow::Exit;
            }
        }

        match event {
            Event::WindowEvent { event: WindowEvent::Resized(physical_size), .. } => {
                println!("New window size received: {}x{}", physical_size.width, physical_size.height);
                if let Ok(mut new_size) = arc_window_size.lock() {
                    *new_size = (physical_size.width, physical_size.height, true);
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            // Keep track of currently pressed keys to send to the rendering thread
            Event::WindowEvent { event: WindowEvent::KeyboardInput {
                    input: KeyboardInput { state: key_state, virtual_keycode: Some(keycode), .. }, .. }, .. } => {

                if let Ok(mut keys) = arc_pressed_keys.lock() {
                    match key_state {
                        Released => {
                            if keys.contains(&keycode) {
                                let i = keys.iter().position(|&k| k == keycode).unwrap();
                                keys.remove(i);
                            }
                        },
                        Pressed => {
                            if !keys.contains(&keycode) {
                                keys.push(keycode);
                            }
                        }
                    }
                }

                // Handle Escape and Q keys separately
                match keycode {
                    Escape => { *control_flow = ControlFlow::Exit; }
                    Q      => { *control_flow = ControlFlow::Exit; }
                    _      => { }
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => {
                // Accumulate mouse movement
                if let Ok(mut position) = arc_mouse_delta.lock() {
                    *position = (position.0 + delta.0 as f32, position.1 + delta.1 as f32);
                }
            }
            _ => { }
        }
    });
}
