<script lang="ts">
  import { onMount, tick } from "svelte";
  import { fade, slide } from "svelte/transition";
  import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";

  let {
    date = $bindable(null),
    time = $bindable(""),
    disabled = false,
  } = $props<{
    date?: string | null;
    time?: string;
    disabled?: boolean;
  }>();

  let showPicker = $state(false);
  let originalHeight = $state<number | null>(null);
  let pickerTriggerElement = $state<HTMLElement | null>(null);
  let opensUpwards = $state(false);

  // Custom Time State
  let customHours = $state("12");
  let customMinutes = $state("00");
  let customPeriod = $state<"AM" | "PM">("PM");

  // Sync initial time -> custom time when opening, or anytime 'time' changes externally
  $effect(() => {
    if (time) {
      const [hStr, mStr] = time.split(':');
      let h = parseInt(hStr, 10);
      if (!isNaN(h)) {
        customPeriod = h >= 12 ? "PM" : "AM";
        if (h === 0) h = 12;
        else if (h > 12) h -= 12;
        customHours = String(h);
      }
      if (mStr) {
        customMinutes = mStr;
      }
    }
  });

  function updateTimeFromCustom() {
    if (!date) return; // Only commit time if a date is selected
    
    let h = parseInt(customHours, 10);
    let m = parseInt(customMinutes, 10);
    
    if (isNaN(h)) h = 12;
    if (isNaN(m)) m = 0;
    
    // Bounds wrapping/clamping
    if (h > 12) h = 12;
    if (h < 1) h = 1;
    if (m > 59) m = 59;
    if (m < 0) m = 0;
    
    let militaryHour = h;
    if (customPeriod === "PM" && h !== 12) militaryHour += 12;
    if (customPeriod === "AM" && h === 12) militaryHour = 0;
    
    time = `${String(militaryHour).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
  }

  function handlePeriodToggle() {
    if (!date) return;
    customPeriod = customPeriod === "AM" ? "PM" : "AM";
    updateTimeFromCustom();
  }

  function formatTimeBlur(type: "hours" | "minutes") {
    if (type === "hours") {
      let h = parseInt(customHours, 10);
      if (isNaN(h) || h < 1) h = 12;
      if (h > 12) h = 12;
      customHours = String(h);
    } else {
      let m = parseInt(customMinutes, 10);
      if (isNaN(m) || m < 0) m = 0;
      if (m > 59) m = 59;
      customMinutes = String(m).padStart(2, '0');
    }
    updateTimeFromCustom();
  }

  // The month currently being viewed in the calendar grid
  let viewDate = $state(date ? new Date(date) : new Date());

  const weekdays = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
  const months = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December"
  ];

  // Helper to format the displayed trigger text
  let displayString = $derived.by(() => {
    if (!date) return "No deadline";
    const d = new Date(date + "T00:00:00");
    const dStr = d.toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric"
    });
    
    if (time) {
      // Just visually formatting HH:MM to 12h or keep as is.
      // Easiest is to parse dummy date
      const fake = new Date(`2000-01-01T${time}`);
      const tStr = fake.toLocaleTimeString("en-US", {
        hour: 'numeric',
        minute: '2-digit'
      });
      return `${dStr} • ${tStr}`;
    }
    return dStr;
  });

  // Calendar Grid generation
  let calendarDays = $derived.by(() => {
    const year = viewDate.getFullYear();
    const month = viewDate.getMonth();
    
    const firstDay = new Date(year, month, 1);
    const lastDay = new Date(year, month + 1, 0);
    
    const startPadding = firstDay.getDay();
    const daysInMonth = lastDay.getDate();
    
    // Previous month padding
    const prevMonthLastDay = new Date(year, month, 0).getDate();
    
    const days = [];
    
    // Add prev month days
    for (let i = startPadding - 1; i >= 0; i--) {
      days.push({
        num: prevMonthLastDay - i,
        isCurrentMonth: false,
        dateStr: `${month === 0 ? year - 1 : year}-${String(month === 0 ? 12 : month).padStart(2, '0')}-${String(prevMonthLastDay - i).padStart(2, '0')}`
      });
    }
    
    // Add current month days
    for (let i = 1; i <= daysInMonth; i++) {
      days.push({
        num: i,
        isCurrentMonth: true,
        dateStr: `${year}-${String(month + 1).padStart(2, '0')}-${String(i).padStart(2, '0')}`
      });
    }
    
    // Add next month padding to complete 6 rows (42 days)
    const remaining = 42 - days.length;
    for (let i = 1; i <= remaining; i++) {
      days.push({
        num: i,
        isCurrentMonth: false,
        dateStr: `${month === 11 ? year + 1 : year}-${String(month === 11 ? 1 : month + 2).padStart(2, '0')}-${String(i).padStart(2, '0')}`
      });
    }
    
    return days;
  });

  async function togglePicker() {
    if (disabled) return;
    if (!showPicker) {
      viewDate = date ? new Date(date + "T00:00:00") : new Date();
      showPicker = true;
      
      // Expand window if too short
      try {
        const appWindow = getCurrentWindow();
        const factor = await appWindow.scaleFactor();
        const physicalSize = await appWindow.innerSize();
        const logicalSize = physicalSize.toLogical(factor);
        
        if (logicalSize.height < 820) {
          originalHeight = logicalSize.height;
          await appWindow.setSize(new LogicalSize(logicalSize.width, 820));
        }
      } catch (e) {
        console.warn("Could not resize window:", e);
      }

      // Check for flip-up logic
      await tick();
      if (pickerTriggerElement) {
        const rect = pickerTriggerElement.getBoundingClientRect();
        const spaceBelow = window.innerHeight - rect.bottom;
        // Popover is around 400px tall with padding/gap
        opensUpwards = spaceBelow < 420;
      }
    } else {
      closePicker();
    }
  }

  async function closePicker() {
    showPicker = false;
    if (originalHeight !== null) {
      try {
        const appWindow = getCurrentWindow();
        const factor = await appWindow.scaleFactor();
        const physicalSize = await appWindow.innerSize();
        const logicalSize = physicalSize.toLogical(factor);
        
        await appWindow.setSize(new LogicalSize(logicalSize.width, originalHeight));
      } catch (e) {
        console.warn("Could not restore window size:", e);
      }
      originalHeight = null;
    }
  }

  function prevMonth() {
    viewDate = new Date(viewDate.getFullYear(), viewDate.getMonth() - 1, 1);
  }

  function nextMonth() {
    viewDate = new Date(viewDate.getFullYear(), viewDate.getMonth() + 1, 1);
  }

  function selectDate(dateStr: string) {
    date = dateStr;
    // Keep picker open so user can adjust time if they want
  }

  function handleClear() {
    date = null;
    time = "";
    closePicker();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && showPicker) {
      closePicker();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="datetime-picker">
  <button 
    type="button" 
    class="picker-trigger input" 
    class:has-value={!!date}
    {disabled}
    onclick={togglePicker}
    bind:this={pickerTriggerElement}
  >
    <span class="icon">📅</span>
    <span class="value">{displayString}</span>
  </button>

  {#if showPicker}
    <div class="calendar-view-backdrop" aria-label="Close picker" role="button" tabindex="0" onclick={closePicker} onkeydown={(e) => e.key === 'Escape' && closePicker()}></div>
    
    <div 
      class="picker-popover" 
      class:opens-upwards={opensUpwards}
      transition:fade={{ duration: 150 }}
    >
      <!-- Header / Nav -->
      <div class="popover-header">
        <button type="button" class="nav-btn" aria-label="Previous month" onclick={prevMonth}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m15 18-6-6 6-6"/></svg>
        </button>
        <div class="current-month-year">
          {months[viewDate.getMonth()]} {viewDate.getFullYear()}
        </div>
        <button type="button" class="nav-btn" aria-label="Next month" onclick={nextMonth}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m9 18 6-6-6-6"/></svg>
        </button>
      </div>

      <!-- Days of Week -->
      <div class="weekdays">
        {#each weekdays as w}
          <div class="weekday">{w}</div>
        {/each}
      </div>

      <!-- Calendar Grid -->
      <div class="days-grid">
        {#each calendarDays as d}
          {@const isSelected = date === d.dateStr}
          {@const isToday = new Date().toLocaleDateString('en-CA') === d.dateStr}
          
          <button 
            type="button"
            class="day-btn" 
            class:current-month={d.isCurrentMonth}
            class:other-month={!d.isCurrentMonth}
            class:selected={isSelected}
            class:today={isToday}
            onclick={() => selectDate(d.dateStr)}
          >
            <div class="day-number">{d.num}</div>
          </button>
        {/each}
      </div>

      <!-- Time Input & Actions -->
      <div class="popover-footer">
        <div class="custom-time-section" class:disabled={!date}>
          <span class="time-icon">⏱</span>
          <div class="time-inputs box-input">
            <input 
              type="number" 
              class="time-num-input" 
              bind:value={customHours} 
              oninput={updateTimeFromCustom}
              onblur={() => formatTimeBlur("hours")}
              min="1" max="12"
              placeholder="12"
              disabled={!date}
            />
            <span class="time-colon">:</span>
            <input 
              type="number" 
              class="time-num-input" 
              bind:value={customMinutes} 
              oninput={updateTimeFromCustom}
              onblur={() => formatTimeBlur("minutes")}
              min="0" max="59"
              placeholder="00"
              disabled={!date}
            />
          </div>
          <button 
            type="button" 
            class="period-toggle" 
            onclick={handlePeriodToggle}
            disabled={!date}
          >
            {customPeriod}
          </button>
        </div>
        
        <button type="button" class="btn btn-ghost btn-sm clear-btn" onclick={handleClear}>
          Clear
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .datetime-picker {
    position: relative;
    width: 100%;
  }

  .picker-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: var(--spacing-sm);
    text-align: left;
    cursor: pointer;
    background: var(--bg-secondary);
    color: var(--text-tertiary); /* Placeholder color */
    transition: all var(--transition-fast);
  }
  
  .picker-trigger:hover:not(:disabled) {
    background: var(--bg-hover);
    border-color: var(--text-tertiary);
  }

  .picker-trigger.has-value {
    color: var(--text-primary);
  }

  .icon {
    font-size: 14px;
    opacity: 0.8;
  }

  .value {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .picker-popover {
    position: absolute;
    top: calc(100% + var(--spacing-xs));
    left: 0;
    width: 280px;
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    z-index: 100;
    padding: var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .picker-popover.opens-upwards {
    top: auto;
    bottom: calc(100% + var(--spacing-xs));
    box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.3);
  }

  /* Header */
  .popover-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .current-month-year {
    font-weight: 600;
    font-size: var(--text-md);
    color: var(--text-primary);
  }

  .nav-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    transition: background 0.1s;
  }

  .nav-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  /* Calendar Grid */
  .weekdays {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    text-align: center;
    font-size: 11px;
    font-weight: 700;
    color: var(--text-tertiary);
    text-transform: uppercase;
    margin-bottom: -4px;
  }

  .days-grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 2px;
  }

  .day-btn {
    aspect-ratio: 1;
    background: transparent;
    border: none;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 13px;
    cursor: pointer;
    color: var(--text-primary);
    transition: all 0.1s;
    position: relative;
  }

  .day-btn:hover {
    background: var(--bg-hover);
  }

  .day-btn.other-month {
    color: var(--text-tertiary);
  }

  .day-btn.today {
    color: var(--accent);
    font-weight: 600;
  }

  .day-btn.today::after {
    content: '';
    position: absolute;
    bottom: 4px;
    left: 50%;
    transform: translateX(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--accent);
  }

  .day-btn.selected {
    background: var(--accent);
    color: white;
    font-weight: 600;
  }

  .day-btn.selected::after {
    background: white;
  }

  /* Custom Time Input & Footer */
  .popover-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: var(--spacing-md);
    border-top: 1px solid var(--border-light);
  }

  .custom-time-section {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    opacity: 1;
    transition: opacity 0.2s;
  }

  .custom-time-section.disabled {
    opacity: 0.4;
    pointer-events: none;
  }

  .time-icon {
    font-size: 14px;
    color: var(--text-secondary);
  }

  .box-input {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    padding: 2px 6px;
    height: 32px;
  }

  .time-num-input {
    width: 24px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-family: var(--font-mono);
    font-size: 13px;
    text-align: center;
    padding: 0;
  }

  .time-num-input:focus {
    outline: none;
  }

  /* Hide number arrows */
  .time-num-input::-webkit-inner-spin-button,
  .time-num-input::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .time-num-input[type=number] {
    -moz-appearance: textfield;
    appearance: textfield;
  }

  .time-colon {
    font-weight: 600;
    color: var(--text-tertiary);
    margin: 0 2px;
  }

  .period-toggle {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-size: 11px;
    font-weight: 700;
    height: 32px;
    width: 36px;
    cursor: pointer;
    transition: all 0.1s;
  }

  .period-toggle:hover {
    background: var(--bg-hover);
    border-color: var(--text-tertiary);
  }

  .clear-btn {
    color: var(--text-secondary);
  }
  
  .clear-btn:hover {
    color: var(--danger);
    background: var(--danger-glow);
  }

  /* Global backdrop handling (similar to CalendarHeader but isolated) */
  :global(.calendar-view-backdrop) {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 99;
  }
</style>
