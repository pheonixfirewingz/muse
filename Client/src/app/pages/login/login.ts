import {ChangeDetectionStrategy, Component, inject, signal} from '@angular/core';
import {MatCardModule} from '@angular/material/card';
import {MatButtonModule} from '@angular/material/button';
import {MatFormFieldModule} from '@angular/material/form-field';
import {
  AbstractControl, AsyncValidatorFn,
  FormControl,
  FormGroup,
  ReactiveFormsModule,
  ValidationErrors,
  Validators
} from '@angular/forms';
import axios from 'axios';
import {MatInput} from '@angular/material/input';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import  {faEye, faEyeSlash} from '@fortawesome/free-solid-svg-icons';
import {Router} from '@angular/router';
import {environment} from '../../../environments/environment';

@Component({
  selector: 'app-login',
  imports: [MatCardModule, MatButtonModule, MatFormFieldModule, ReactiveFormsModule, MatInput, FaIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './login.html',
  styleUrl: './login.css'
})
export class Login {
  private router = inject(Router);
  public static readonly supportedAPIVersion: number[] = [0,1,0];
  protected isRegisterMode:boolean = false;
  hide = signal(true);

  clickEvent(event: MouseEvent) {
    this.hide.set(!this.hide());
    event.stopPropagation();
  }

  getVisIcon() {
    return this.hide() ? faEyeSlash : faEye;
  }

  protected readonly login = new FormGroup({
    username: new FormControl('local_checks',[Validators.required]),
    password: new FormControl('tuh6y6Q8N5q*tF4^vhx&@fPE8s',[Validators.required, Validators.minLength(12)]),
  });

  protected readonly register = new FormGroup({
    origin: new FormControl(this.getStoredOrigin() || '', [Validators.required, Login.originCheck]),
    username: new FormControl('', [Validators.required]),
    email: new FormControl('', [Validators.required, Validators.email]),
    password: new FormControl('', [Validators.required, Validators.minLength(12)]),
    confirmPassword: new FormControl('', [Validators.required])
  });

  toggleMode():void {
    this.isRegisterMode = !this.isRegisterMode;
  }

  get username():string {
    return (this.isRegisterMode ? this.register.get('username')?.value : this.login.get('username')?.value) as string;
  }

  get password():string {
    return (this.isRegisterMode ? this.register.get('password')?.value : this.login.get('password')?.value) as string;
  }

  get email():string {
    return this.register.get('email')?.value as string;
  }

  get confirmPassword():string {
    return this.register.get('confirmPassword')?.value as string;
  }

  // Helper methods for localStorage
  private getStoredOrigin(): string | null {
    try {
      return localStorage.getItem('apiOrigin');
    } catch (error) {
      console.warn('localStorage not available:', error);
      return null;
    }
  }

  private setStoredOrigin(origin: string): void {
    try {
      localStorage.setItem('apiOrigin', origin);
    } catch (error) {
      console.warn('Could not save origin to localStorage:', error);
    }
  }

  static originCheck(): AsyncValidatorFn {
    return async (control: AbstractControl): Promise<ValidationErrors | null> => {
      const value = control.value;
      if (!value)
        return null;

      let api: boolean;
      try {
        const data = await axios.get(`${value}/api/muse_server_version`);
        console.info(data);
        //TODO: check server api version as we may need to as in a legacy way in the future
        api = true;
      } catch (_) {
        api = false;
      }
      // Fixed: return null when valid, error object when invalid
      return api ? null : { invalidOrigin: true };
    }
  }

  async sendLoginRequest(): Promise<void> {
    if (!this.login.valid) {
      console.log('Form is invalid, not sending request');
      return;
    }
    try {
      const json = await (await axios.post(`${environment.apiUrl}/api/login`, {
        username: this.username,
        password: this.password,
      }, { headers: {
          'Content-Type': 'application/json',
        }, withCredentials: true,})).data;

      if (json.success) {
        console.info(json.message);
        localStorage.setItem('authToken',json.token)
        await this.router.navigate(['/app']);
      } else {
        console.error('Login failed:', json.message);
      }
    } catch (error) {
      console.error('Login error:', error);
    }
  }

  async signupRequest(): Promise<void> {
    if (!this.register.valid) {
      console.log('Form is invalid, not sending request');
      return;
    }
    try {
      const json = await (await axios.post(`${environment.apiUrl}/api/register`, {
        username: this.username,
        email: this.email,
        password: this.password,
        confirmPassword: this.confirmPassword
      }, { headers: {
          'Content-Type': 'application/json',
        }})).data;

      if (json.success) {
        this.isRegisterMode = false;
      } else {
        console.error('Registration failed:', json.message);
      }
    } catch (error) {
      console.error('Registration error:', error);
    }
  }
}
