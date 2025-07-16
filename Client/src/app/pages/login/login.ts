import {ChangeDetectionStrategy, Component, inject, signal} from '@angular/core';
import {MatCardModule} from '@angular/material/card';
import {MatButtonModule} from '@angular/material/button';
import {MatFormFieldModule} from '@angular/material/form-field';
import {
  FormControl,
  FormGroup,
  ReactiveFormsModule,
  Validators
} from '@angular/forms';
import {MatInput} from '@angular/material/input';
import {FaIconComponent} from '@fortawesome/angular-fontawesome';
import  {faEye, faEyeSlash} from '@fortawesome/free-solid-svg-icons';
import {Router} from '@angular/router';
import {environment} from '../../../environments/environment';
import { fetchWithAuth } from '../../app';
import { ApiService } from '../../services/api.service';

// API Response interfaces
interface LoginResponse {
  success: boolean;
  token?: string;
  message?: string;
}

interface RegisterResponse {
  success: boolean;
  message?: string;
}

@Component({
  selector: 'app-login',
  imports: [MatCardModule, MatButtonModule, MatFormFieldModule, ReactiveFormsModule, MatInput, FaIconComponent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './login.html',
  styleUrl: './login.scss'
})
export class Login {
  private router = inject(Router);
  private apiService = inject(ApiService);
  protected isRegisterMode:boolean = false;
  hide = signal(true);

  constructor() {
    // Configure the API service to use the environment settings
    this.apiService.configure({
      host: '127.0.0.1',
      port: 8000,
      useHttps: true // Try HTTPS first, fallback to HTTP if needed
    });
    
    // Test connection to determine available protocols
    this.testConnection();
  }

  async testConnection(): Promise<void> {
    try {
      const result = await this.apiService.testConnection().toPromise();
      console.log('Connection test results:', result);
      
      if (result?.https) {
        console.log('✅ HTTPS is available and enabled');
        this.apiService.configure({ useHttps: true });
      } else if (result?.http) {
        console.log('✅ HTTP is available (HTTPS not enabled)');
        this.apiService.configure({ useHttps: false });
      } else {
        console.log('❌ No connection available');
      }
    } catch (error) {
      console.log('❌ Connection test failed:', error);
    }
  }

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

  async sendLoginRequest(): Promise<void> {
    if (!this.login.valid) {
      console.log('Form is invalid, not sending request');
      return;
    }
    try {
      const response = await this.apiService.post<LoginResponse>('/api/login', {
        username: this.username,
        password: this.password,
      }).toPromise();
      
      if (response && response.success) {
        localStorage.setItem('authToken', response.token || '');
        await this.router.navigate(['/app']);
      } else {
        console.error('Login failed:', response?.message);
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
      const response = await this.apiService.post<RegisterResponse>('/api/register', {
        username: this.username,
        email: this.email,
        password: this.password,
        confirmPassword: this.confirmPassword
      }).toPromise();

      if (response && response.success) {
        this.isRegisterMode = false;
      } else {
        console.error('Registration failed:', response?.message);
      }
    } catch (error) {
      console.error('Registration error:', error);
    }
  }
}
