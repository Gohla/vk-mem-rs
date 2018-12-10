extern crate ash;
#[macro_use]
extern crate bitflags;

use ash::vk::Handle;
use std::mem;

pub mod ffi;

#[derive(Debug, Clone)]
pub struct Allocator {
    pub(crate) internal: ffi::VmaAllocator,
}

#[derive(Debug, Clone)]
pub struct AllocatorPool {
    pub(crate) internal: ffi::VmaPool,
}

impl Default for AllocatorPool {
    fn default() -> Self {
        AllocatorPool {
            internal: unsafe { mem::zeroed() },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Allocation {
    pub(crate) internal: ffi::VmaAllocation,
    pub(crate) info: ffi::VmaAllocationInfo,
}

#[derive(Debug, Clone)]
pub struct AllocatorCreateInfo {
    pub physical_device: ash::vk::PhysicalDevice,
    pub device: ash::vk::Device,
}

#[inline]
fn ffi_to_result(result: ffi::VkResult) -> ash::vk::Result {
    ash::vk::Result::from_raw(result)
}

fn allocation_create_info_to_ffi(info: &AllocationCreateInfo) -> ffi::VmaAllocationCreateInfo {
    let mut create_info: ffi::VmaAllocationCreateInfo = unsafe { mem::zeroed() };
    create_info.usage = match &info.usage {
        MemoryUsage::Unknown => ffi::VmaMemoryUsage_VMA_MEMORY_USAGE_UNKNOWN,
        MemoryUsage::GpuOnly => ffi::VmaMemoryUsage_VMA_MEMORY_USAGE_GPU_ONLY,
        MemoryUsage::CpuOnly => ffi::VmaMemoryUsage_VMA_MEMORY_USAGE_CPU_ONLY,
        MemoryUsage::CpuToGpu => ffi::VmaMemoryUsage_VMA_MEMORY_USAGE_CPU_TO_GPU,
        MemoryUsage::GpuToCpu => ffi::VmaMemoryUsage_VMA_MEMORY_USAGE_GPU_TO_CPU,
    };
    create_info.flags = info.flags.bits();
    create_info.requiredFlags = info.required_flags.as_raw();
    create_info.preferredFlags = info.preferred_flags.as_raw();
    create_info.memoryTypeBits = info.memory_type_bits;
    create_info.pool = info.pool.internal;
    create_info.pUserData = info.user_data;
    create_info
}

#[derive(Debug, Clone)]
pub enum MemoryUsage {
    Unknown,
    GpuOnly,
    CpuOnly,
    CpuToGpu,
    GpuToCpu,
}

bitflags! {
    pub struct AllocationCreateFlags: u32 {
        const NONE = 0x00000000;
        const DEDICATED_MEMORY = 0x00000001;
        const NEVER_ALLOCATE = 0x00000002;
        const MAPPED = 0x00000004;
        const CAN_BECOME_LOST = 0x00000008;
        const CAN_MAKE_OTHER_LOST = 0x00000010;
        const USER_DATA_COPY_STRING = 0x00000020;
        const UPPER_ADDRESS = 0x00000040;
        const STRATEGY_BEST_FIT = 0x00010000;
        const STRATEGY_WORST_FIT = 0x00020000;
        const STRATEGY_FIRST_FIT = 0x00040000;
        const STRATEGY_MIN_MEMORY = 0x00010000;
        const STRATEGY_MIN_TIME = 0x00040000;
        const STRATEGY_MIN_FRAGMENTATION = 0x00020000;
        const STRATEGY_MASK = 0x00010000 | 0x00020000 | 0x00040000;
    }
}

#[derive(Debug, Clone)]
pub struct AllocationCreateInfo {
    pub usage: MemoryUsage,
    pub flags: AllocationCreateFlags,
    pub required_flags: ash::vk::MemoryPropertyFlags,
    pub preferred_flags: ash::vk::MemoryPropertyFlags,
    pub memory_type_bits: u32,
    pub pool: AllocatorPool,
    pub user_data: *mut ::std::os::raw::c_void,
}

impl Default for AllocationCreateInfo {
    fn default() -> Self {
        AllocationCreateInfo {
            usage: MemoryUsage::Unknown,
            flags: AllocationCreateFlags::NONE,
            required_flags: ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            preferred_flags: ash::vk::MemoryPropertyFlags::DEVICE_LOCAL,
            memory_type_bits: 0,
            pool: AllocatorPool::default(),
            user_data: ::std::ptr::null_mut(),
        }
    }
}

impl Allocator {
    pub fn new(create_info: &AllocatorCreateInfo) -> Self {
        let mut ffi_create_info: ffi::VmaAllocatorCreateInfo = unsafe { mem::zeroed() };
        ffi_create_info.physicalDevice =
            create_info.physical_device.as_raw() as ffi::VkPhysicalDevice;
        ffi_create_info.device = create_info.device.as_raw() as ffi::VkDevice;
        let mut internal: ffi::VmaAllocator = unsafe { mem::zeroed() };
        let result = ffi_to_result(unsafe {
            ffi::vmaCreateAllocator(
                &ffi_create_info as *const ffi::VmaAllocatorCreateInfo,
                &mut internal,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("new - error occurred! {}", result));
            }
        }

        Allocator { internal }
    }

    // TODO: vmaGetPhysicalDeviceProperties
    /*
    pub fn vmaGetPhysicalDeviceProperties(
        allocator: VmaAllocator,
        ppPhysicalDeviceProperties: *mut *const VkPhysicalDeviceProperties,
    );
    */
    
    // TODO: vmaGetMemoryProperties
    /*
    pub fn vmaGetMemoryProperties(
        allocator: VmaAllocator,
        ppPhysicalDeviceMemoryProperties: *mut *const VkPhysicalDeviceMemoryProperties,
    );
    */

    // TODO: vmaGetMemoryTypeProperties
    /*
    pub fn vmaGetMemoryTypeProperties(
        allocator: VmaAllocator,
        memoryTypeIndex: u32,
        pFlags: *mut VkMemoryPropertyFlags,
    );
    */

    pub fn set_current_frame_index(&self, frame_index: u32) {
        unsafe {
            ffi::vmaSetCurrentFrameIndex(
                self.internal,
                frame_index,
            );
        }
    }

    pub fn calculate_stats(&self) -> ffi::VmaStats {
        let mut vma_stats: ffi::VmaStats = unsafe { mem::zeroed() };
        unsafe {
            ffi::vmaCalculateStats(
                self.internal,
                &mut vma_stats,
            );
        }
        vma_stats
    }

    pub fn build_stats_string(&self, detailed_map: bool) -> String {
        let mut stats_string: *mut ::std::os::raw::c_char = ::std::ptr::null_mut();
        unsafe {
            ffi::vmaBuildStatsString(self.internal, &mut stats_string, if detailed_map { 1 } else { 0 });
        }
        if stats_string.is_null() {
            String::new()
        } else {
            let result = unsafe { std::ffi::CStr::from_ptr(stats_string).to_string_lossy().into_owned() };
            unsafe { ffi::vmaFreeStatsString(self.internal, stats_string); }
            result
        }
    }

    // TODO: vmaFindMemoryTypeIndex
    /*
    pub fn vmaFindMemoryTypeIndex(
        allocator: VmaAllocator,
        memoryTypeBits: u32,
        pAllocationCreateInfo: *const VmaAllocationCreateInfo,
        pMemoryTypeIndex: *mut u32,
    ) -> VkResult;
    */

    // TODO: vmaFindMemoryTypeIndexForBufferInfo
    /*
    pub fn vmaFindMemoryTypeIndexForBufferInfo(
        allocator: VmaAllocator,
        pBufferCreateInfo: *const VkBufferCreateInfo,
        pAllocationCreateInfo: *const VmaAllocationCreateInfo,
        pMemoryTypeIndex: *mut u32,
    ) -> VkResult;
    */

    // TODO: vmaFindMemoryTypeIndexForImageInfo
    /*
    pub fn vmaFindMemoryTypeIndexForImageInfo(
        allocator: VmaAllocator,
        pImageCreateInfo: *const VkImageCreateInfo,
        pAllocationCreateInfo: *const VmaAllocationCreateInfo,
        pMemoryTypeIndex: *mut u32,
    ) -> VkResult;
    */

    // TODO: vmaCreatePool
    /*
    pub fn vmaCreatePool(
        allocator: VmaAllocator,
        pCreateInfo: *const VmaPoolCreateInfo,
        pPool: *mut VmaPool,
    ) -> VkResult;
    */

    pub fn destroy_pool(&mut self, pool: &AllocatorPool) {
        unsafe {
            ffi::vmaDestroyPool(
                self.internal,
                pool.internal,
            );
        }
    }

    pub fn get_pool_stats(&self, pool: &AllocatorPool) -> ffi::VmaPoolStats {
        let mut pool_stats: ffi::VmaPoolStats = unsafe { mem::zeroed() };
        unsafe {
            ffi::vmaGetPoolStats(
                self.internal,
                pool.internal,
                &mut pool_stats,
            );
        }
        pool_stats
    }

    pub fn make_pool_allocations_lost(&mut self, pool: &mut AllocatorPool) -> usize {
        let mut lost_count: usize = 0;
        unsafe {
            ffi::vmaMakePoolAllocationsLost(
                self.internal,
                pool.internal,
                &mut lost_count,
            );
        }
        lost_count
    }

    pub fn check_pool_corruption(&self, pool: &AllocatorPool) {
        let result =
            ffi_to_result(unsafe { ffi::vmaCheckPoolCorruption(self.internal, pool.internal) });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("check_pool_corruption - error occurred! {}", result));
            }
        }
    }

    // TODO: vmaAllocateMemory
    /*
    pub fn vmaAllocateMemory(
        allocator: VmaAllocator,
        pVkMemoryRequirements: *const VkMemoryRequirements,
        pCreateInfo: *const VmaAllocationCreateInfo,
        pAllocation: *mut VmaAllocation,
        pAllocationInfo: *mut VmaAllocationInfo,
    ) -> VkResult;
    */

    // TODO: vmaAllocateMemoryForBuffer
    /*
    pub fn vmaAllocateMemoryForBuffer(
        allocator: VmaAllocator,
        buffer: VkBuffer,
        pCreateInfo: *const VmaAllocationCreateInfo,
        pAllocation: *mut VmaAllocation,
        pAllocationInfo: *mut VmaAllocationInfo,
    ) -> VkResult;
    */

    // TODO: vmaAllocateMemoryForImage
    /*
    pub fn vmaAllocateMemoryForImage(
        allocator: VmaAllocator,
        image: VkImage,
        pCreateInfo: *const VmaAllocationCreateInfo,
        pAllocation: *mut VmaAllocation,
        pAllocationInfo: *mut VmaAllocationInfo,
    ) -> VkResult;
    */

    pub fn free_memory(&mut self, allocation: &Allocation) {
        unsafe {
            ffi::vmaFreeMemory(
                self.internal,
                allocation.internal,
            );
        }
    }

    pub fn resize_allocation(&mut self, allocation: &Allocation, new_size: usize) {
        let result = ffi_to_result(unsafe {
            ffi::vmaResizeAllocation(
                self.internal,
                allocation.internal,
                new_size as ffi::VkDeviceSize,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("resize_allocation - error occurred! {}", result));
            }
        }
    }

    pub fn get_allocation_info(&mut self, allocation: &mut Allocation) {
        unsafe {
            ffi::vmaGetAllocationInfo(
                self.internal,
                allocation.internal,
                &mut allocation.info
            )
        }
    }

    pub fn touch_allocation(&mut self, allocation: &Allocation) -> bool {
        let result = unsafe {
            ffi::vmaTouchAllocation(
                self.internal,
                allocation.internal,
            )
        };
        if result == 1 {
            true
        } else {
            false
        }
    }

    pub fn set_allocation_user_data(&mut self, allocation: &Allocation, user_data: *mut ::std::os::raw::c_void) {
        unsafe {
            ffi::vmaSetAllocationUserData(
                self.internal,
                allocation.internal,
                user_data,
            );
        }
    }

    pub fn create_lost_allocation(&mut self) -> Allocation {
        let mut allocation: Allocation = unsafe { mem::zeroed() };
        unsafe {
            ffi::vmaCreateLostAllocation(
                self.internal,
                &mut allocation.internal,
            );
        }
        allocation
    }

    pub fn map_memory(&mut self, allocation: &Allocation) -> *mut u8 {
        let mut mapped_data: *mut ::std::os::raw::c_void = ::std::ptr::null_mut();
        let result = ffi_to_result(unsafe {
            ffi::vmaMapMemory(
                self.internal,
                allocation.internal,
                &mut mapped_data,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("map_memory - error occurred! {}", result));
            }
        }
        mapped_data as *mut u8
        //unsafe { std::slice::from_raw_parts(mapped_data as *mut u8, 1) }
    }

    pub fn unmap_memory(&mut self, allocation: &Allocation) {
        unsafe {
            ffi::vmaUnmapMemory(
                self.internal,
                allocation.internal,
            );
        }
    }

    pub fn flush_allocation(&mut self, allocation: &Allocation, offset: usize, size: usize) {
        unsafe {
            ffi::vmaFlushAllocation(
                self.internal,
                allocation.internal,
                offset as ffi::VkDeviceSize,
                size as ffi::VkDeviceSize,
            );
        }
    }

    pub fn invalidate_allocation(&mut self, allocation: &Allocation, offset: usize, size: usize) {
        unsafe {
            ffi::vmaInvalidateAllocation(
                self.internal,
                allocation.internal,
                offset as ffi::VkDeviceSize,
                size as ffi::VkDeviceSize,
            );
        }
    }

    pub fn check_corruption(&self, memory_types: ash::vk::MemoryPropertyFlags) {
        let result =
            ffi_to_result(unsafe { ffi::vmaCheckCorruption(self.internal, memory_types.as_raw()) });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("check_corruption - error occurred! {}", result));
            }
        }
    }

    // TODO: vmaDefragment
    /*
    pub fn vmaDefragment(
        allocator: VmaAllocator,
        pAllocations: *mut VmaAllocation,
        allocationCount: usize,
        pAllocationsChanged: *mut VkBool32,
        pDefragmentationInfo: *const VmaDefragmentationInfo,
        pDefragmentationStats: *mut VmaDefragmentationStats,
    ) -> VkResult;
    */

    pub fn bind_buffer_memory(&mut self, buffer: ash::vk::Buffer, allocation: &Allocation) {
        let result = ffi_to_result(unsafe {
            ffi::vmaBindBufferMemory(
                self.internal,
                allocation.internal,
                buffer.as_raw() as ffi::VkBuffer,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("bind_buffer_memory - error occurred! {}", result));
            }
        }
    }

    pub fn bind_image_memory(&mut self, image: ash::vk::Image, allocation: &Allocation) {
        let result = ffi_to_result(unsafe {
            ffi::vmaBindImageMemory(
                self.internal,
                allocation.internal,
                image.as_raw() as ffi::VkImage,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("bind_image_memory - error occurred! {}", result));
            }
        }
    }

    pub fn create_buffer(
        &mut self,
        buffer_info: &ash::vk::BufferCreateInfo,
        allocation_info: &AllocationCreateInfo,
    ) -> (ash::vk::Buffer, Allocation) {
        let buffer_create_info = unsafe {
            mem::transmute::<&ash::vk::BufferCreateInfo, &ffi::VkBufferCreateInfo>(buffer_info)
        };
        let allocation_create_info = allocation_create_info_to_ffi(&allocation_info);
        let mut buffer: ffi::VkBuffer = unsafe { mem::zeroed() };
        let mut allocation: Allocation = unsafe { mem::zeroed() };

        let result = ffi_to_result(unsafe {
            ffi::vmaCreateBuffer(
                self.internal,
                buffer_create_info,
                &allocation_create_info,
                &mut buffer,
                &mut allocation.internal,
                &mut allocation.info,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("create_buffer - error occurred! {}", result));
            }
        }
        (ash::vk::Buffer::from_raw(buffer as u64), allocation)
    }

    pub fn destroy_buffer(&mut self, buffer: ash::vk::Buffer, allocation: &Allocation) {
        unsafe {
            ffi::vmaDestroyBuffer(
                self.internal,
                buffer.as_raw() as ffi::VkBuffer,
                allocation.internal,
            );
        }
    }

    pub fn create_image(
        &mut self,
        image_info: &ash::vk::ImageCreateInfo,
        allocation_info: &AllocationCreateInfo,
    ) -> (ash::vk::Image, Allocation) {
        let image_create_info = unsafe {
            mem::transmute::<&ash::vk::ImageCreateInfo, &ffi::VkImageCreateInfo>(image_info)
        };
        let allocation_create_info = allocation_create_info_to_ffi(&allocation_info);
        let mut image: ffi::VkImage = unsafe { mem::zeroed() };
        let mut allocation: Allocation = unsafe { mem::zeroed() };
        let result = ffi_to_result(unsafe {
            ffi::vmaCreateImage(
                self.internal,
                image_create_info,
                &allocation_create_info,
                &mut image,
                &mut allocation.internal,
                &mut allocation.info,
            )
        });
        match result {
            ash::vk::Result::SUCCESS => {
                // Success
            }
            _ => {
                panic!(format!("create_image - error occurred! {}", result));
            }
        }
        (ash::vk::Image::from_raw(image as u64), allocation)
    }

    pub fn destroy_image(&mut self, image: ash::vk::Image, allocation: &Allocation) {
        unsafe {
            ffi::vmaDestroyImage(
                self.internal,
                image.as_raw() as ffi::VkImage,
                allocation.internal,
            );
        }
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        if !self.internal.is_null() {
            unsafe {
                ffi::vmaDestroyAllocator(self.internal);
            }
        }
    }
}
