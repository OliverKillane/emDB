# Helper scripts for debugging lare stack frames with combi

define show_frame_sizes
    set $frame_num = 0
    set $prev_sp = 0
    set $curr_frame = 0

    while $curr_frame < 100
        frame $curr_frame
        set $curr_sp = $rsp

        if $frame_num > 0
            set $frame_size = $curr_sp - $prev_sp
            printf "Frame %d: Size = %d bytes, Function = ", $frame_num - 1, $frame_size
        else
            printf "Frame %d: Function = ", $frame_num
        end

        set $prev_sp = $curr_sp
        set $frame_num = $frame_num + 1
        set $curr_frame = $curr_frame + 1
    end
end

define show_large_frames
    set $frame_num = 0
    set $prev_sp = 0
    set $curr_frame = 0

    while $curr_frame < 100
        frame $curr_frame
        set $curr_sp = $rsp
        set $func_name = ""
        if $frame_num > 0
            set $frame_size = $curr_sp - $prev_sp
            if $frame_size > 8192
                printf "⚠️ LARGE STACK FRAME! Frame %d: %d bytes, Function: ", $frame_num - 1, $frame_size
                info symbol $pc
            end
        end

        set $prev_sp = $curr_sp
        set $frame_num = $frame_num + 1
        set $curr_frame = $curr_frame + 1
    end
end
