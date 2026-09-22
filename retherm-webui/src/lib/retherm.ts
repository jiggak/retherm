export interface RethermSettings {
   presets: Preset[];
   away: AwayMode | null;
   schedule: Schedule[];
}

export type PresetName = 'Home' | 'Sleep' | 'Away';

export interface Preset {
   name: PresetName
}

export interface AwayMode {
   preset: PresetName;
   timeout: string;
}

export type WeekDayRange = 'EveryDay' | 'WeekDays' | 'WeekEnd';
export type WeekDay = 'Mon' | 'Tue' | 'Wed' | 'Thur' | 'Fri' | 'Sat' | 'Sun';

export interface SetPoint {
   time: string;
   preset: PresetName;
}

export interface Schedule {
   days_of_week: WeekDayRange | WeekDay[];
   set_points: SetPoint[];
}
